# Veon Programming Language

Veon is a small interpreted language implemented in Rust. It now supports:

- Scalars: numbers, booleans, strings, null
- Collections: arrays with concatenation and indexing
- Control flow: if/else, while, for (desugared to while)
- Functions: declarations, parameters, returns, recursion, and closures
- Classes and instances: fields, methods (including `init` initializers), property access and mutation, `this`
- Logical operators: `and`, `or`, plus arithmetic and comparison operators

## Running code

1. Build and run a Veon program file:

```bash
cargo run -- main.vp
```

2. To run a different script, pass its path instead of `main.vp`.

Programs print the last evaluated value, making it easy to inspect results from expressions or function calls.

## Included sample

`main.vp` demonstrates classes, functions, recursion, arrays, and loops:

```text
// Sample Veon program demonstrating language features
class Counter {
  fun init(start) { this.value = start; }
  fun inc() { this.value = this.value + 1; return this.value; }
}

fun fib(n) {
  if (n < 2) { return n; }
  return fib(n - 1) + fib(n - 2);
}

fun greet(first, second) {
  return "Hello " + first + " & " + second + "!";
}

let counter = Counter(3);
let next = counter.inc();
let numbers = [1, 1, 2, 3];
let total = 0;
for (let i = 0; i < 4; i = i + 1) {
  total = total + numbers[i];
}

// chaining features together: functions, objects, arrays, loops
let summary = greet("Ada", "Grace") + " fib(6)=" + fib(6);
summary;
total + next;
```

Running it prints the result of the final expression after exercising all the constructs above.
