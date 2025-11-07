;; ABOUTME: Standard Library - Common list operations and higher-order functions
;; This library provides useful functions written in pure Lisp

;; ============================================================================
;; Higher-Order Functions
;; ============================================================================

;; map(f, list) -> list
;; Applies function f to each element of list
;; Example: (map (lambda (x) (* x 2)) '(1 2 3)) => (2 4 6)
(define (map f lst)
  (if (empty? lst)
      '()
      (cons (f (car lst))
            (map f (cdr lst)))))

;; filter(pred, list) -> list
;; Keeps only elements matching predicate
;; Example: (filter (lambda (x) (> x 2)) '(1 2 3 4 5)) => (3 4 5)
(define (filter pred lst)
  (if (empty? lst)
      '()
      (if (pred (car lst))
          (cons (car lst) (filter pred (cdr lst)))
          (filter pred (cdr lst)))))

;; reduce(f, init, list) -> value
;; Accumulates values using function f starting with init
;; Example: (reduce + 0 '(1 2 3 4)) => 10
(define (reduce f init lst)
  (if (empty? lst)
      init
      (reduce f (f init (car lst)) (cdr lst))))

;; compose(f, g) -> function
;; Combines two functions: (f . g)(x) = f(g(x))
;; Example: (define double (lambda (x) (* x 2)))
;;          (define inc (lambda (x) (+ x 1)))
;;          ((compose double inc) 5) => 12
(define (compose f g)
  (lambda (x) (f (g x))))

;; partial(f, arg) -> function
;; Partial function application
;; Example: (define add5 (partial + 5))
;;          (add5 10) => 15
(define (partial f arg)
  (lambda (x) (f arg x)))

;; ============================================================================
;; List Utilities
;; ============================================================================

;; reverse(list) -> list
;; Reverses a list
;; Example: (reverse '(1 2 3)) => (3 2 1)
(define (reverse-helper lst acc)
  (if (empty? lst)
      acc
      (reverse-helper (cdr lst) (cons (car lst) acc))))

(define (reverse lst)
  (reverse-helper lst '()))

;; append(list1, list2) -> list
;; Concatenates two lists
;; Example: (append '(1 2) '(3 4)) => (1 2 3 4)
(define (append lst1 lst2)
  (if (empty? lst1)
      lst2
      (cons (car lst1) (append (cdr lst1) lst2))))

;; member(x, list) -> bool
;; Checks if element is in list
;; Example: (member 2 '(1 2 3)) => #t
(define (member x lst)
  (if (empty? lst)
      #f
      (if (= x (car lst))
          #t
          (member x (cdr lst)))))

;; nth(n, list) -> element
;; Gets element at index n (0-based)
;; Example: (nth 0 '(1 2 3)) => 1
(define (nth n lst)
  (if (= n 0)
      (car lst)
      (nth (- n 1) (cdr lst))))

;; last(list) -> element
;; Gets the last element of a list
;; Example: (last '(1 2 3)) => 3
(define (last lst)
  (if (empty? (cdr lst))
      (car lst)
      (last (cdr lst))))

;; take(n, list) -> list
;; Gets first n elements
;; Example: (take 2 '(1 2 3 4)) => (1 2)
(define (take n lst)
  (if (= n 0)
      '()
      (cons (car lst) (take (- n 1) (cdr lst)))))

;; drop(n, list) -> list
;; Skips first n elements
;; Example: (drop 2 '(1 2 3 4)) => (3 4)
(define (drop n lst)
  (if (= n 0)
      lst
      (drop (- n 1) (cdr lst))))

;; zip(list1, list2) -> list of pairs
;; Combines two lists into pairs
;; Example: (zip '(1 2 3) '(a b c)) => ((1 a) (2 b) (3 c))
(define (zip lst1 lst2)
  (if (empty? lst1)
      '()
      (cons (list (car lst1) (car lst2))
            (zip (cdr lst1) (cdr lst2)))))

;; ============================================================================
;; Predicate Functions
;; ============================================================================

;; all(pred, list) -> bool
;; Checks if all elements match predicate
;; Example: (all (lambda (x) (> x 0)) '(1 2 3)) => #t
(define (all pred lst)
  (if (empty? lst)
      #t
      (if (pred (car lst))
          (all pred (cdr lst))
          #f)))

;; any(pred, list) -> bool
;; Checks if any element matches predicate
;; Example: (any (lambda (x) (> x 2)) '(1 2 3)) => #t
(define (any pred lst)
  (if (empty? lst)
      #f
      (if (pred (car lst))
          #t
          (any pred (cdr lst)))))

;; count(pred, list) -> number
;; Counts elements matching predicate
;; Example: (count (lambda (x) (> x 2)) '(1 2 3 4 5)) => 3
(define (count pred lst)
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
;; Example: (range 0 5) => (0 1 2 3 4)
(define (range start end)
  (if (>= start end)
      '()
      (cons start (range (+ start 1) end))))

;; ============================================================================
;; Math Utilities
;; ============================================================================

;; abs(x) -> number
;; Absolute value
;; Example: (abs -5) => 5
(define (abs x)
  (if (< x 0)
      (- x)
      x))

;; min(x, y) -> number
;; Minimum of two values
;; Example: (min 3 5) => 3
(define (min x y)
  (if (< x y) x y))

;; max(x, y) -> number
;; Maximum of two values
;; Example: (max 3 5) => 5
(define (max x y)
  (if (> x y) x y))

;; square(x) -> number
;; Squares a number
;; Example: (square 5) => 25
(define (square x)
  (* x x))

;; cube(x) -> number
;; Cubes a number
;; Example: (cube 3) => 27
(define (cube x)
  (* x x x))

;; even?(x) -> bool
;; Checks if number is even
;; Example: (even? 4) => #t
(define (even? x)
  (= (% x 2) 0))

;; odd?(x) -> bool
;; Checks if number is odd
;; Example: (odd? 3) => #t
(define (odd? x)
  (not (even? x)))

;; sum(list) -> number
;; Sum of list elements
;; Example: (sum '(1 2 3 4)) => 10
(define (sum lst)
  (reduce + 0 lst))

;; product(list) -> number
;; Product of list elements
;; Example: (product '(1 2 3 4)) => 24
(define (product lst)
  (reduce * 1 lst))

;; factorial(n) -> number
;; Factorial function (recursive)
;; Example: (factorial 5) => 120
(define (factorial n)
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
