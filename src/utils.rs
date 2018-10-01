fn with<T>(mv:T, mf:fn(&T)->()) -> T { mf(&mv); mv }
fn withed<T>(mv:T, mf:fn(T)->T) -> T { mf(mv) }
fn co<U,V>(a:U, f:fn(U)->V) -> V { f(a) }

fn ireduce<T>(a:T, i:Iterable, f:fn(T)->T) -> T {
    for e in i { f(a,e) };
    a
}
