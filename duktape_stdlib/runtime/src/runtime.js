(function (root) {

    const io = require('io'),
        utils = require('utils'),
        _slice = Array.prototype.slice,
        _has = Object.prototype.hasOwnProperty;



    function Console() {}

    function write(w, l, args) {
        var format = utils.format.apply(void 0, args);
        w.write(format + '\n');
    }

    Object.assign(Console.prototype, {
        log() {
            write(io.stdout, 'debug', _slice.call(arguments));
        },
        info() {
            write(io.stdout, 'info', _slice.call(arguments));
        },
        warn() {
            write(io.stderr, 'warn', _slice.call(arguments));
        },
        error() {
            write(io.stderr, 'error', _slice.call(arguments))
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
})