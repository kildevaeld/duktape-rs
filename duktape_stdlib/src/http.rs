use super::io::ReaderKey;
use duktape::prelude::*;
use duktape::{
    class,
    error::{ErrorKind, Result, ResultExt},
    Key,
};
use duktape_modules::{require, CJSContext};
use reqwest::{header::HeaderMap, header::HeaderName, Client, Method, Response, Url};
use std::str::FromStr;

pub static HTTP: &'static [u8] = include_bytes!("../runtime/dist/http.js");

struct ClientKey;

impl Key for ClientKey {
    type Value = Client;
}

fn options_to_request(options: &Object, client: &Client) -> Result<Response> {
    let (method, url) = get_method(options)?;
    let can_have_body = method == Method::POST || method == Method::PATCH || method == Method::PUT;
    let mut req = client.request(method, url);

    if options.has("headers") {
        let mut headers = HeaderMap::new();
        for (key, value) in options.get::<_, Object>("headers")?.iter() {
            headers.insert(
                HeaderName::from_bytes(key.as_bytes()).unwrap(),
                value.get::<&str>()?.parse().unwrap(),
            );
        }
    }

    if options.has("body") && can_have_body {
        let body = options.get::<_, Ref>("body")?;
        req = match body.get_type() {
            Type::Buffer => req.body(body.get::<&[u8]>()?.to_vec()),
            _ => req.body(body.to_string()),
        };
    }

    req.send()
        .chain_err(|| ErrorKind::Error("could not connect".to_string()))
}

fn get_method(o: &Object) -> Result<(Method, Url)> {
    let method: &str = o.get("method")?;
    if !o.has("url") {
        return Err(ErrorKind::TypeError(format!("missing url property")).into());
    }

    let url: Url = match o.get::<_, &str>("url")?.parse() {
        Ok(u) => u,
        Err(_) => return Err(ErrorKind::TypeError(format!("invalid url")).into()),
    };

    match Method::from_str(method) {
        Ok(m) => Ok((m, url)),
        Err(e) => Err(ErrorKind::TypeError(format!("{}", e)).into()),
    }
}

fn push_response(ctx: &Context, resp: Response) -> Result<Object> {
    let o: Object = ctx.create()?;

    let headers: Object = ctx.create()?;
    for (key, value) in resp.headers().iter() {
        let value = value
            .to_str()
            .chain_err(|| ErrorKind::TypeError("could not get header".to_string()))?;
        if headers.has(key) {
            let array = headers.get::<_, Array>(key)?;
            array.push(value)?;
        } else {
            headers.set(key, vec![value]);
        }
    }

    o.set("url", resp.url().as_str())
        .set("headers", headers)
        .set("status", resp.status().as_u16());

    if let Some(address) = resp.remote_addr() {
        o.set("remoteAddress", address.to_string());
    } else {
        o.set("remoteAddress", ());
    }

    ctx.require("http")
        .unwrap()
        .get::<_, Ref>("ResponseReader")?
        .push();
    ctx.construct(0)?;
    duktape::class::get_instance(ctx, -1, move |this| {
        this.data_mut().insert::<ReaderKey>(Box::new(resp));
        Ok(())
    })?;

    o.set("body", ctx.getp::<Ref>()?);

    // let mut body = Vec::new();
    // match resp.read_to_end(&mut body) {
    //     Ok(_) => (),
    //     Err(e) => return Err(ErrorKind::Error(format!("could not read body: {}", e)).into()),
    // };

    // o.set("body", body.as_slice());

    Ok(o)
}

// fn request(method: Method, ctx: &Context, _: &mut class::Instance) -> Result<i32> {
//     let o = if ctx.is(Type::String, 0) {
//         let o = ctx.create::<Object>()?;
//         o.set("url", ctx.get::<&str>(0)?);
//         o
//     } else {
//         ctx.get::<Object>(0)?
//     };

//     o.set("method", method.to_string());

//     let this = ctx.push_this().getp::<Object>()?;
//     this.call::<_, _, Ref>("request", o)?.push();
//     Ok(1)
// }

fn build_client_class<'a>() -> class::Builder<'a> {
    let mut b = class::build();
    b.constructor((1, |_ctx: &Context, instance: &mut class::Instance| {
        let client = Client::new();
        instance.data_mut().insert::<ClientKey>(client);
        Ok(0)
    }))
    .method(
        "request",
        (1, |ctx: &Context, instance: &mut class::Instance| {
            let options: Object = ctx.get(0)?;
            let client = instance.data().get::<ClientKey>().unwrap();
            let resp = options_to_request(&options, client)?;
            ctx.push(push_response(ctx, resp)?)?;
            Ok(1)
        }),
    );
    // .method(
    //     "get",
    //     (1, |ctx: &Context, this: &mut class::Instance| {
    //         request(Method::GET, ctx, this)
    //     }),
    // )
    // .method(
    //     "post",
    //     (1, |ctx: &Context, this: &mut class::Instance| {
    //         request(Method::POST, ctx, this)
    //     }),
    // )
    // .method(
    //     "put",
    //     (1, |ctx: &Context, this: &mut class::Instance| {
    //         request(Method::PUT, ctx, this)
    //     }),
    // )
    // .method(
    //     "patch",
    //     (1, |ctx: &Context, this: &mut class::Instance| {
    //         request(Method::PATCH, ctx, this)
    //     }),
    // )
    // .method(
    //     "del",
    //     (1, |ctx: &Context, this: &mut class::Instance| {
    //         request(Method::DELETE, ctx, this)
    //     }),
    // );

    b
}

fn build_body_class(ctx: &Context) -> Result<class::Builder> {
    let mut b = class::build();

    let ctor = ctx
        .get_global_string("require")
        .push_string("io")
        .call(1)?
        .get_prop_string(-1, "Reader")
        .getp::<Function>()?;

    ctx.pop(1);

    b.inherit(ctor);

    Ok(b)
}

pub fn init_http(ctx: &Context) -> Result<i32> {
    let exports: Object = ctx.create()?;

    exports.set("Client", build_client_class());
    exports.set("ResponseReader", build_body_class(ctx)?);

    let module: Object = ctx.get(-1)?;
    module.set("exports", exports);

    require::eval_module(ctx, HTTP, &module).unwrap();

    module.get::<_, Ref>("exports")?.push();

    Ok(1)
}
