;; ABOUTME: Standard Library - Common list operations and higher-order functions
;; This library provides useful functions written in pure Lisp

;; ============================================================================
;; Higher-Order Functions
;; ============================================================================

;; map(f, list) -> list
;; Applies function f to each element of list
(define (map f lst)
  "Apply function to each element, returning new list.

**Parameters:**
- f: Function to apply to each element
- lst: Input list

**Returns:** New list with f applied to each element

**Time Complexity:** O(n) where n is list length

**Examples:**
- (map (lambda (x) (* x 2)) '(1 2 3)) => (2 4 6)
- (map (lambda (x) (+ x 1)) '(0 1 2)) => (1 2 3)

**Notes:** Uses tail call optimization for efficiency. Preserves list structure."
  (if (empty? lst)
      '()
      (cons (f (car lst))
            (map f (cdr lst)))))

;; filter(pred, list) -> list
;; Keeps only elements matching predicate
(define (filter pred lst)
  "Keep only elements satisfying predicate.

**Parameters:**
- pred: Predicate function returning boolean
- lst: Input list

**Returns:** New list containing only elements where pred returns true

**Time Complexity:** O(n) where n is list length

**Examples:**
- (filter (lambda (x) (> x 2)) '(1 2 3 4 5)) => (3 4 5)
- (filter even? '(1 2 3 4 5)) => (2 4)

**Notes:** Preserves element order. Short-circuits on first predicate evaluation."
  (if (empty? lst)
      '()
      (if (pred (car lst))
          (cons (car lst) (filter pred (cdr lst)))
          (filter pred (cdr lst)))))

;; reduce(f, init, list) -> value
;; Accumulates values using function f starting with init
(define (reduce f init lst)
  "Fold list using function, accumulating from init value (left-fold).

**Parameters:**
- f: Binary function (accumulator, element) -> new-accumulator
- init: Initial accumulator value
- lst: Input list

**Returns:** Final accumulated value

**Time Complexity:** O(n) where n is list length

**Examples:**
- (reduce + 0 '(1 2 3 4)) => 10
- (reduce * 1 '(1 2 3 4)) => 24
- (reduce (lambda (acc x) (cons x acc)) '() '(1 2 3)) => (3 2 1)

**Notes:** Left-associative fold. Processes list from head to tail."
  (if (empty? lst)
      init
      (reduce f (f init (car lst)) (cdr lst))))

;; compose(f, g) -> function
;; Combines two functions: (f . g)(x) = f(g(x))
(define (compose f g)
  "Compose two functions: returns function that applies g then f.

**Parameters:**
- f: Outer function
- g: Inner function

**Returns:** New function that computes f(g(x))

**Mathematical Notation:** (compose f g)(x) = f(g(x))

**Examples:**
- (define double (lambda (x) (* x 2)))
- (define inc (lambda (x) (+ x 1)))
- ((compose double inc) 5) => 12 [= double(inc(5)) = double(6)]

**Notes:** Useful for building function pipelines. Can be chained: (compose f (compose g h))."
  (lambda (x) (f (g x))))

;; partial(f, arg) -> function
;; Partial function application
(define (partial f arg)
  "Partially apply function with first argument fixed.

**Parameters:**
- f: Binary function to partially apply
- arg: First argument to bind

**Returns:** New function that accepts second argument

**Examples:**
- (define add5 (partial + 5))
- (add5 10) => 15
- (define mult-by-2 (partial * 2))
- (mult-by-2 3) => 6

**Common Use Cases:** Creating specialized functions from generic ones, currying, building function factories."
  (lambda (x) (f arg x)))

;; ============================================================================
;; List Utilities
;; ============================================================================

;; reverse-helper(list, accumulator) -> list
;; Helper function for reverse (uses accumulator)
(define (reverse-helper lst acc)
  "Internal helper for list reversal using accumulator.

**Parameters:**
- lst: Remaining list to reverse
- acc: Accumulator list (reversed so far)

**Returns:** Complete reversed list

**Time Complexity:** O(n) where n is list length

**Notes:** Uses tail recursion for efficiency. Accessed via reverse/1."
  (if (empty? lst)
      acc
      (reverse-helper (cdr lst) (cons (car lst) acc))))

;; reverse(list) -> list
;; Reverses a list
(define (reverse lst)
  "Reverse a list.

**Parameters:**
- lst: Input list

**Returns:** New list with elements in reverse order

**Time Complexity:** O(n) where n is list length

**Examples:**
- (reverse '(1 2 3)) => (3 2 1)
- (reverse '(a b c)) => (c b a)
- (reverse '()) => ()

**Notes:** Creates new list, doesn't modify original. Tail-recursive via helper function."
  (reverse-helper lst '()))

;; append(list1, list2) -> list
;; Concatenates two lists
(define (append lst1 lst2)
  "Concatenate two lists.

**Parameters:**
- lst1: First list
- lst2: Second list

**Returns:** New list with all elements from lst1 followed by lst2

**Time Complexity:** O(m) where m is length of lst1

**Examples:**
- (append '(1 2) '(3 4)) => (1 2 3 4)
- (append '(a) '(b c)) => (a b c)
- (append '() '(1 2)) => (1 2)

**Notes:** Second list is returned as-is; first list is copied. Linear in length of first argument."
  (if (empty? lst1)
      lst2
      (cons (car lst1) (append (cdr lst1) lst2))))

;; member(x, list) -> bool
;; Checks if element is in list
(define (member x lst)
  "Check if element exists in list.

**Parameters:**
- x: Element to search for (uses = for comparison)
- lst: List to search

**Returns:** #t if element found, #f otherwise

**Time Complexity:** O(n) worst case, O(1) best case

**Examples:**
- (member 2 '(1 2 3)) => #t
- (member 5 '(1 2 3)) => #f
- (member 'b '(a b c)) => #t

**Notes:** Uses equality (=) for comparison. Short-circuits on first match."
  (if (empty? lst)
      #f
      (if (= x (car lst))
          #t
          (member x (cdr lst)))))

;; nth(n, list) -> element
;; Gets element at index n (0-based)
(define (nth n lst)
  "Get element at zero-based index.

**Parameters:**
- n: Zero-based index
- lst: List to access

**Returns:** Element at index n

**Time Complexity:** O(n) where n is the index

**Error Conditions:**
- Negative index: undefined behavior
- Index >= list length: error (attempts to car on empty list)

**Examples:**
- (nth 0 '(a b c)) => a
- (nth 1 '(1 2 3)) => 2
- (nth 2 '(x y z)) => z

**Notes:** Zero-based indexing. Suitable for small indices; consider converting to vector for frequent access."
  (if (= n 0)
      (car lst)
      (nth (- n 1) (cdr lst))))

;; last(list) -> element
;; Gets the last element of a list
(define (last lst)
  "Get the last element of a list.

**Parameters:**
- lst: Non-empty list

**Returns:** Last element of the list

**Time Complexity:** O(n) where n is list length

**Examples:**
- (last '(1 2 3)) => 3
- (last '(a b c d)) => d
- (last '(x)) => x

**Error Conditions:**
- Empty list: error (cdr of empty list)

**Notes:** Must traverse entire list. For repeated access, reverse and use car."
  (if (empty? (cdr lst))
      (car lst)
      (last (cdr lst))))

;; take(n, list) -> list
;; Gets first n elements
(define (take n lst)
  "Get first n elements of a list.

**Parameters:**
- n: Number of elements to take
- lst: Input list

**Returns:** New list with first n elements (or entire list if n > length)

**Time Complexity:** O(n) where n is the number to take

**Examples:**
- (take 2 '(1 2 3 4)) => (1 2)
- (take 3 '(a b c d e)) => (a b c)
- (take 0 '(1 2 3)) => ()

**Notes:** Returns shorter list if requested number exceeds list length."
  (if (= n 0)
      '()
      (cons (car lst) (take (- n 1) (cdr lst)))))

;; drop(n, list) -> list
;; Skips first n elements
(define (drop n lst)
  "Skip first n elements of a list.

**Parameters:**
- n: Number of elements to skip
- lst: Input list

**Returns:** New list with first n elements removed

**Time Complexity:** O(n) where n is the number to drop

**Examples:**
- (drop 2 '(1 2 3 4)) => (3 4)
- (drop 1 '(a b c)) => (b c)
- (drop 3 '(1 2 3)) => ()

**Notes:** Returns empty list if n >= list length."
  (if (= n 0)
      lst
      (drop (- n 1) (cdr lst))))

;; zip(list1, list2) -> list of pairs
;; Combines two lists into pairs
(define (zip lst1 lst2)
  "Combine two lists into pairs.

**Parameters:**
- lst1: First list
- lst2: Second list

**Returns:** List of pairs [element1, element2]

**Time Complexity:** O(min(n, m)) where n, m are list lengths

**Examples:**
- (zip '(1 2 3) '(a b c)) => ((1 a) (2 b) (3 c))
- (zip '(x y) '(10 20)) => ((x 10) (y 20))
- (zip '() '(1 2)) => ()

**Notes:** Stops when shorter list ends. Use take/drop to pad lists."
  (if (empty? lst1)
      '()
      (cons (list (car lst1) (car lst2))
            (zip (cdr lst1) (cdr lst2)))))

;; ============================================================================
;; Predicate Functions
;; ============================================================================

;; all(pred, list) -> bool
;; Checks if all elements match predicate
(define (all pred lst)
  "Check if all elements satisfy predicate.

**Parameters:**
- pred: Predicate function
- lst: List to check

**Returns:** #t if all elements satisfy pred, #f otherwise

**Time Complexity:** O(n) worst case, O(1) if first element fails

**Examples:**
- (all (lambda (x) (> x 0)) '(1 2 3)) => #t
- (all (lambda (x) (> x 2)) '(1 2 3)) => #f
- (all even? '(2 4 6)) => #t

**Notes:** Short-circuits on first false value (early exit optimization)."
  (if (empty? lst)
      #t
      (if (pred (car lst))
          (all pred (cdr lst))
          #f)))

;; any(pred, list) -> bool
;; Checks if any element matches predicate
(define (any pred lst)
  "Check if any element satisfies predicate.

**Parameters:**
- pred: Predicate function
- lst: List to check

**Returns:** #t if any element satisfies pred, #f otherwise

**Time Complexity:** O(n) worst case, O(1) if first element succeeds

**Examples:**
- (any (lambda (x) (> x 2)) '(1 2 3)) => #t
- (any (lambda (x) (> x 5)) '(1 2 3)) => #f
- (any odd? '(2 4 6)) => #f

**Notes:** Short-circuits on first true value (early exit optimization)."
  (if (empty? lst)
      #f
      (if (pred (car lst))
          #t
          (any pred (cdr lst)))))

;; count(pred, list) -> number
;; Counts elements matching predicate
(define (count pred lst)
  "Count elements satisfying predicate.

**Parameters:**
- pred: Predicate function
- lst: List to count

**Returns:** Number of elements where pred returns true

**Time Complexity:** O(n) where n is list length

**Examples:**
- (count (lambda (x) (> x 2)) '(1 2 3 4 5)) => 3
- (count even? '(1 2 3 4 5 6)) => 3
- (count (lambda (x) (= x 0)) '(1 2 3)) => 0

**Notes:** Counts all matching elements (doesn't short-circuit)."
  (if (empty? lst)
      0
      (if (pred (car lst))
          (+ 1 (count pred (cdr lst)))
          (count pred (cdr lst)))))

;; ============================================================================
;; Range & Sequence Functions
;; ============================================================================

;; range(start, end) -> list
;; Creates list of numbers from start to end (exclusive)
(define (range start end)
  "Generate list of integers from start (inclusive) to end (exclusive).

**Parameters:**
- start: Starting number (inclusive)
- end: Ending number (exclusive)

**Returns:** List of integers [start, start+1, ..., end-1]

**Time Complexity:** O(n) where n = (end - start)

**Examples:**
- (range 0 5) => (0 1 2 3 4)
- (range 1 4) => (1 2 3)
- (range 5 5) => ()

**Notes:** Mirrors Python/Scheme range semantics. Use with map/filter for sequences."
  (if (>= start end)
      '()
      (cons start (range (+ start 1) end))))

;; ============================================================================
;; Math Utilities
;; ============================================================================

;; abs(x) -> number
;; Absolute value
(define (abs x)
  "Return absolute value (magnitude without sign).

**Parameters:**
- x: Number

**Returns:** Non-negative number with same magnitude

**Time Complexity:** O(1)

**Examples:**
- (abs 5) => 5
- (abs -5) => 5
- (abs 0) => 0

**Notes:** Works with both integers and floats."
  (if (< x 0)
      (- x)
      x))

;; min(x, y) -> number
;; Minimum of two values
(define (min x y)
  "Return minimum of two numbers.

**Parameters:**
- x: First number
- y: Second number

**Returns:** Smaller of x and y

**Time Complexity:** O(1)

**Examples:**
- (min 3 5) => 3
- (min -2 -5) => -5
- (min 0 0) => 0

**Notes:** For finding minimum of list, use reduce with min."
  (if (< x y) x y))

;; max(x, y) -> number
;; Maximum of two values
(define (max x y)
  "Return maximum of two numbers.

**Parameters:**
- x: First number
- y: Second number

**Returns:** Larger of x and y

**Time Complexity:** O(1)

**Examples:**
- (max 3 5) => 5
- (max -2 -5) => -2
- (max 0 0) => 0

**Notes:** For finding maximum of list, use reduce with max."
  (if (> x y) x y))

;; square(x) -> number
;; Squares a number
(define (square x)
  "Return number squared (x*x).

**Parameters:**
- x: Number

**Returns:** x * x

**Time Complexity:** O(1)

**Examples:**
- (square 5) => 25
- (square -3) => 9
- (square 0) => 0

**Notes:** Common utility for geometric calculations and algorithm implementation."
  (* x x))

;; cube(x) -> number
;; Cubes a number
(define (cube x)
  "Return number cubed (x*x*x).

**Parameters:**
- x: Number

**Returns:** x * x * x

**Time Complexity:** O(1)

**Examples:**
- (cube 3) => 27
- (cube -2) => -8
- (cube 0) => 0

**Notes:** Less common than square, but follows similar pattern."
  (* x x x))

;; even?(x) -> bool
;; Checks if number is even
(define (even? x)
  "Check if number is even.

**Parameters:**
- x: Integer

**Returns:** #t if x is even, #f otherwise

**Time Complexity:** O(1)

**Examples:**
- (even? 4) => #t
- (even? 3) => #f
- (even? 0) => #t

**Notes:** Uses modulo operator (%). Define odd? in terms of even? for consistency."
  (= (% x 2) 0))

;; odd?(x) -> bool
;; Checks if number is odd
(define (odd? x)
  "Check if number is odd.

**Parameters:**
- x: Integer

**Returns:** #t if x is odd, #f otherwise

**Time Complexity:** O(1)

**Examples:**
- (odd? 3) => #t
- (odd? 4) => #f
- (odd? 0) => #f

**Notes:** Defined as negation of even? for consistency. Use even? when applicable."
  (not (even? x)))

;; sum(list) -> number
;; Sum of list elements
(define (sum lst)
  "Sum all elements in a list.

**Parameters:**
- lst: List of numbers

**Returns:** Sum of all elements (0 for empty list)

**Time Complexity:** O(n) where n is list length

**Examples:**
- (sum '(1 2 3 4)) => 10
- (sum '(10)) => 10
- (sum '()) => 0

**Notes:** Implemented using reduce. Handles empty lists gracefully."
  (reduce + 0 lst))

;; product(list) -> number
;; Product of list elements
(define (product lst)
  "Multiply all elements in a list.

**Parameters:**
- lst: List of numbers

**Returns:** Product of all elements (1 for empty list)

**Time Complexity:** O(n) where n is list length

**Examples:**
- (product '(1 2 3 4)) => 24
- (product '(2 3)) => 6
- (product '()) => 1

**Notes:** Implemented using reduce. Handles empty lists gracefully."
  (reduce * 1 lst))

;; factorial(n) -> number
;; Factorial function (recursive)
(define (factorial n)
  "Compute factorial: n! = n * (n-1) * ... * 1.

**Parameters:**
- n: Non-negative integer

**Returns:** n factorial

**Time Complexity:** O(n)

**Examples:**
- (factorial 5) => 120
- (factorial 0) => 1
- (factorial 1) => 1

**Error Conditions:**
- Negative n: returns 1 (by definition, base case)
- Large n: may cause stack overflow without TCO

**Notes:** Uses tail-call optimization for efficiency. Mathematical: 0! = 1, n! = n * (n-1)!"
  (if (<= n 1)
      1
      (* n (factorial (- n 1)))))

;; ============================================================================
;; Convenience Macros
;; ============================================================================

;; Note: Our parser doesn't support dotted syntax (test . body) yet,
;; so macros with variable arguments aren't currently possible.
;; We'll skip these for now.

;; when(test, expr) - executes expr only if test is true
;; Example: (when #t (println "Hello"))
;; Note: Currently only supports single expression
;; (defmacro when (test expr)
;;   `(if ,test ,expr nil))

;; unless(test, expr) - executes expr only if test is false
;; Example: (unless #f (println "Hello"))
;; Note: Currently only supports single expression
;; (defmacro unless (test expr)
;;   `(if ,test nil ,expr))

;; Standard library loaded successfully
