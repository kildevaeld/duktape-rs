const methods = ['get', 'post', 'put', 'patch', 'head', 'option'];
const slice = Array.prototype.slice;

var defaultClient;

function request(method) {
    return function (urlOrOptions, optionsOrNull) {

        var options = urlOrOptions
        if (typeof urlOrOptions == 'string') {
            options = optionsOrNull || {};
            options.url = urlOrOptions;
        }
        options.method = method.toUpperCase();

        return this.request(options);
    }
}

for (var i = 0, ii = methods.length; i < ii; i++) {
    module.exports[methods[i]] = (function (method) {
        return function () {
            if (!defaultClient)
                defaultClient = new module.exports.Client();
            return defaultClient[method].apply(defaultClient, slice.call(arguments));
        }
    })(methods[i]);

    module.exports.Client.prototype[methods[i]] = request(methods[i]);
}

Object.freeze(module.exports);