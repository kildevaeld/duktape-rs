use duktape::prelude::*;
use duktape::{
    self, class,
    error::{ErrorKind, Result},
    Key,
};
use reqwest::{Client, Method, RequestBuilder, Response, Url};
use std::str::FromStr;

struct ClientKey;

impl Key for ClientKey {
    type Value = Client;
}

fn options_to_request(options: &Object, client: &Client) -> Result<Response> {
    let (method, url) = get_method(options)?;
    let can_have_body =
        (method == Method::POST || method == Method::PATCH || method == Method::PUT);
    let req = client.request(method, url);

    if options.has("body") && can_have_body {
        let body = options.get::<_, Ref>("body")?.to_string();
    }

    Err(ErrorKind::TypeError("".to_owned()).into())
}

fn get_method(o: &Object) -> Result<(Method, Url)> {
    let method: &str = o.get("method")?;

    if !o.has("url") {
        return Err(ErrorKind::TypeError(format!("missing url property")).into());
    }

    let url: Url = match o.get::<_, &str>("url")?.parse() {
        Ok(u) => u,
        Err(e) => return Err(ErrorKind::TypeError(format!("invalid url")).into()),
    };

    match Method::from_str(method) {
        Ok(m) => Ok((m, url)),
        Err(e) => Err(ErrorKind::TypeError(format!("{}", e)).into()),
    }
}

fn build_client_class<'a>() -> class::Builder<'a> {
    let mut b = class::build();
    b.constructor((1, |_ctx: &Context, instance: &mut class::Instance| {
        let client = Client::new();

        instance.data_mut().insert::<ClientKey>(client);
        Ok(0)
    }))
    .method(
        "request",
        |ctx: &Context, instance: &mut class::Instance| {
            let options: Object = ctx.get(0)?;
            let client = instance.data().get::<ClientKey>().unwrap();

            Ok(0)
        },
    );
    b
}

pub fn init_http(ctx: &Context) -> Result<i32> {
    let module: Object = ctx.create()?;

    module.set("Client", build_client_class());

    ctx.push(module)?;

    Ok(1)
}
