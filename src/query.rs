
struct Query {
    query: String,


}

struct Filter {
    field: String,
    equality: String,
    value: Sting,
    value_type: String
}

enum Equality {
    Equal = "=",
    Lt = "<",
    Gt = ">"
}