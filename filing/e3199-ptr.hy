package main

struct bag {
    counts map[string]int
}

fn main() {
    var p *bag
    fmt.println(p == nil)
    p.counts["k"] = 1
    fmt.println(len(p.counts))
}
