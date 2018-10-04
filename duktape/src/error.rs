error_chain!{

    errors {
        Unknown {
            description("unknown error")
            display("unknown error")
        }
        InsufficientMemory {
            description("Insufficient Memory")
            display("Insufficient Memory")
        }
        TypeError(message: String) {
            description("TypeError")
            display("Type error: {}", message)
        }
        ReferenceError(message: String) {
            description("ReferenceError")
            display("Reference error: {}", message)
        }
    }


}
