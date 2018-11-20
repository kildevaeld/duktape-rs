const http = require('http');

// const client = new http.Client();


// var resp = client.request({
//     method: 'GET',
//     url: 'http://md5.jsontest.com/?text=example_text'
// });

// const decoder = new TextDecoder('utf-8');

// var body = JSON.parse(decoder.decode(resp.body));

// console.log(body);

try {
    resp = http.get("https://google.com")
} catch (e) {
    console.error(e);
}

const decoder = new TextDecoder('utf-8');
// var buf;
// while ((buf = resp.body.read())) {
//     //console.log(buf);
//     console.log(decoder.decode(buf));
// }

console.log(decoder.decode(resp.body.readAll()));

console.log(resp.headers['content-type'], resp.remoteAddress);