if (typeof global === 'undefined') {
    (function () {
        var global = new Function('return this;')();
        Object.defineProperty(global, 'global', {
            value: global,
            writable: true,
            enumerable: false,
            configurable: true
        });
    })();
}
if (typeof Object.assign === 'undefined') {
    Object.defineProperty(Object, 'assign', {
        value: function (target) {
            var i, n, j, m, k;
            var source, keys;
            var gotError;
            var pendingError;

            if (target == null) {
                throw new Exception('target null or undefined');
            }

            for (i = 1, n = arguments.length; i < n; i++) {
                source = arguments[i];
                if (source == null) {
                    continue;  // null or undefined
                }
                source = Object(source);
                keys = Object.keys(source);  // enumerable own keys

                for (j = 0, m = keys.length; j < m; j++) {
                    k = keys[j];
                    try {
                        target[k] = source[k];
                    } catch (e) {
                        if (!gotError) {
                            gotError = true;
                            pendingError = e;
                        }
                    }
                }
            }

            if (gotError) {
                throw pendingError;
            }
        }, writable: true, enumerable: false, configurable: true
    });
}
