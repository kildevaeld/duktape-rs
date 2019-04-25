(function (root) {

    const
        _slice = Array.prototype.slice,
        _has = Object.prototype.hasOwnProperty;

    function toString(data) {
        if (data && typeof data.toString === 'function') {
            return data.toString();
        }

        return String(data);
    }

    const formatters = {
        s: toString,
        i: toString,
        d: toString,
        O: function O(data) {
            if (data && data instanceof Error) {
                return data.toString();
            } else if (typeof data === 'function') {
                return data.toString();
            }
            return JSON.stringify(data);
        }
    };

    function Console() { }

    Console.format = function () {
        var args = _slice.call(arguments);
        if ('string' !== typeof args[0]) {
            // anything else let's inspect with %O
            args.unshift('%O');
        }
        var index = 0;
        args[0] = args[0].replace(/%([a-zA-Z%])/g, function (match, format) {
            // if we encounter an escaped % then don't increase the array index
            if (match === '%%') return match;
            index++;
            var formatter = formatters[format];
            if ('function' === typeof formatter) {
                var val = args[index];
                match = formatter.call(this, val);
                // now we need to remove `args[index]` since it's inlined in the `format`
                args.splice(index, 1);
                index--;
            }
            return match;
        });
        return args.join(' ');
    };

    function write(w, l, args) {
        var format = Console.format.apply(void 0, args);
        w.write(format + '\n');
    }

    Object.assign(Console.prototype, {
        log() {
            write(process.stdout, 'debug', _slice.call(arguments));
        },
        info() {
            write(process.stdout, 'info', _slice.call(arguments));
        },
        warn() {
            write(process.stderr, 'warn', _slice.call(arguments));
        },
        error() {
            write(process.stderr, 'error', _slice.call(arguments))
        }
    });


    root.Console = Console;

    root.console = new Console();
    root.console = new Proxy(root.console, {
        get: function (t, p) {
            if (typeof t[p] === 'function') {
                return t[p];
            }
            return utils.noop;
        }
    });
})(global);