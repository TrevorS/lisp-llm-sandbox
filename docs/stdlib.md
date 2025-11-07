# Standard Library Reference

Quick reference for all functions available in the standard library.

## Higher-Order Functions

### map(f, list) -> list
Applies function f to each element of list.
```lisp
(map (lambda (x) (* x 2)) '(1 2 3))
=> (2 4 6)
```

### filter(pred, list) -> list
Keeps only elements matching predicate.
```lisp
(filter (lambda (x) (> x 2)) '(1 2 3 4 5))
=> (3 4 5)
```

### reduce(f, init, list) -> value
Accumulates values using function f starting with init.
```lisp
(reduce + 0 '(1 2 3 4))
=> 10
```

### compose(f, g) -> function
Combines two functions: (f . g)(x) = f(g(x)).
```lisp
(define double (lambda (x) (* x 2)))
(define inc (lambda (x) (+ x 1)))
((compose double inc) 5)
=> 12  ; (5 + 1) * 2
```

### partial(f, arg) -> function
Partial function application.
```lisp
(define add5 (partial + 5))
(add5 10)
=> 15
```

## List Utilities

### reverse(list) -> list
Reverses a list.
```lisp
(reverse '(1 2 3))
=> (3 2 1)
```

### append(list1, list2) -> list
Concatenates two lists.
```lisp
(append '(1 2) '(3 4))
=> (1 2 3 4)
```

### member(x, list) -> bool
Checks if element is in list.
```lisp
(member 2 '(1 2 3))
=> #t
```

### nth(n, list) -> element
Gets element at index n (0-based).
```lisp
(nth 0 '(10 20 30))
=> 10
```

### last(list) -> element
Gets the last element of a list.
```lisp
(last '(1 2 3))
=> 3
```

### take(n, list) -> list
Gets first n elements.
```lisp
(take 2 '(1 2 3 4))
=> (1 2)
```

### drop(n, list) -> list
Skips first n elements.
```lisp
(drop 2 '(1 2 3 4))
=> (3 4)
```

### zip(list1, list2) -> list of pairs
Combines two lists into pairs.
```lisp
(zip '(1 2 3) '(a b c))
=> ((1 a) (2 b) (3 c))
```

## Predicate Functions

### all(pred, list) -> bool
Checks if all elements match predicate.
```lisp
(all (lambda (x) (> x 0)) '(1 2 3))
=> #t
```

### any(pred, list) -> bool
Checks if any element matches predicate.
```lisp
(any (lambda (x) (> x 2)) '(1 2 3))
=> #t
```

### count(pred, list) -> number
Counts elements matching predicate.
```lisp
(count (lambda (x) (> x 2)) '(1 2 3 4 5))
=> 3
```

## Sequence Generation

### range(start, end) -> list
Creates list of numbers from start to end (exclusive).
```lisp
(range 0 5)
=> (0 1 2 3 4)
```

## Math Utilities

### abs(x) -> number
Absolute value.
```lisp
(abs -5)
=> 5
```

### min(x, y) -> number
Minimum of two values.
```lisp
(min 3 5)
=> 3
```

### max(x, y) -> number
Maximum of two values.
```lisp
(max 3 5)
=> 5
```

### square(x) -> number
Squares a number.
```lisp
(square 5)
=> 25
```

### cube(x) -> number
Cubes a number.
```lisp
(cube 3)
=> 27
```

### even?(x) -> bool
Checks if number is even.
```lisp
(even? 4)
=> #t
```

### odd?(x) -> bool
Checks if number is odd.
```lisp
(odd? 3)
=> #t
```

### sum(list) -> number
Sum of list elements.
```lisp
(sum '(1 2 3 4))
=> 10
```

### product(list) -> number
Product of list elements.
```lisp
(product '(1 2 3 4))
=> 24
```

### factorial(n) -> number
Factorial function.
```lisp
(factorial 5)
=> 120
```

## Built-in Functions

All the standard built-in functions are also available:

### Arithmetic
- `+`, `-`, `*`, `/`, `%`

### Comparison
- `=`, `<`, `>`, `<=`, `>=`

### Logic
- `and`, `or`, `not`

### List Operations
- `cons`, `car`, `cdr`, `list`, `length`, `empty?`

### Type Predicates
- `number?`, `string?`, `list?`, `nil?`, `symbol?`, `bool?`

### I/O
- `print`, `println`

### Error Handling
- `error`, `error?`, `error-msg`

## Complex Examples

### Find average of a list
```lisp
(define (average lst)
  (/ (sum lst) (length lst)))

(average '(1 2 3 4 5))
=> 3
```

### Square all numbers in a list
```lisp
(map square '(1 2 3 4 5))
=> (1 4 9 16 25)
```

### Get even numbers from a list
```lisp
(filter even? '(1 2 3 4 5 6))
=> (2 4 6)
```

### Sum of squares
```lisp
(sum (map square '(1 2 3 4 5)))
=> 55
```

### Check if list is sorted
```lisp
(define (sorted? lst)
  (if (empty? (cdr lst))
      #t
      (if (<= (car lst) (car (cdr lst)))
          (sorted? (cdr lst))
          #f)))

(sorted? '(1 2 3 4 5))
=> #t
```

### Count elements greater than n
```lisp
(define (count-greater-than n lst)
  (count (lambda (x) (> x n)) lst))

(count-greater-than 3 '(1 2 3 4 5))
=> 2
```
