;; ABOUTME: String utilities - Character and string manipulation functions
;; This library provides functions for working with strings

;; ============================================================================
;; String Transformation
;; ============================================================================

;;; Capitalize first character of string, lowercase the rest.
;;;
;;; **Parameters:**
;;; - s: Input string
;;;
;;; **Returns:** String with first character capitalized
;;;
;;; **Time Complexity:** O(n) where n is string length
;;;
;;; **Examples:**
;;; - (string-capitalize "hello") => "Hello"
;;; - (string-capitalize "WORLD") => "World"
;;;
;;; **Notes:** Built from string-upper, string-lower, and substring primitives.
(define (string-capitalize s)
  (if (string-empty? s)
      s
      (if (= (string-length s) 1)
          (string-upper s)
          (string-append (string-upper (substring s 0 1))
                        (string-lower (substring s 1 (string-length s)))))))

;;; Concatenate all strings in a list into a single string.
;;;
;;; **Parameters:**
;;; - lst: List of strings
;;;
;;; **Returns:** Single concatenated string
;;;
;;; **Time Complexity:** O(n) where n is total length of all strings
;;;
;;; **Examples:**
;;; - (string-concat '("hello" " " "world")) => "hello world"
;;; - (string-concat '()) => ""
;;;
;;; **Notes:** Handles empty list by returning empty string.
(define (string-concat lst)
  (if (nil? lst)
      ""
      (list->string lst)))

;;; Reverse a string.
;;;
;;; **Parameters:**
;;; - s: Input string
;;;
;;; **Returns:** Reversed string
;;;
;;; **Time Complexity:** O(n) where n is string length
;;;
;;; **Examples:**
;;; - (string-reverse "hello") => "olleh"
;;; - (string-reverse "racecar") => "racecar"
;;;
;;; **Notes:** Converts to list, reverses, converts back to string.
(define (string-reverse s)
  (list->string (reverse (string->list s))))

;;; Tail-recursive helper for string-repeat with accumulator.
;;;
;;; **Parameters:**
;;; - s: String to repeat
;;; - count: Remaining repetitions
;;; - acc: Accumulated result
;;;
;;; **Returns:** Accumulated string after count repetitions
;;;
;;; **Notes:** Internal helper function. Do not call directly.
(define (string-repeat-helper s count acc)
  (if (<= count 0)
      acc
      (string-repeat-helper s (- count 1) (string-append acc s))))

;;; Repeat string n times.
;;;
;;; **Parameters:**
;;; - s: String to repeat
;;; - n: Number of repetitions (non-negative integer)
;;;
;;; **Returns:** Repeated string
;;;
;;; **Time Complexity:** O(n*m) where n is repeat count, m is string length
;;;
;;; **Examples:**
;;; - (string-repeat "ab" 3) => "ababab"
;;; - (string-repeat "x" 0) => ""
;;;
;;; **Notes:** Uses tail-recursive helper with accumulator for efficiency.
(define (string-repeat s n)
  (if (<= n 0)
      ""
      (string-repeat-helper s n "")))

;; ============================================================================
;; String Parsing
;; ============================================================================

;;; Split string into words by whitespace.
;;;
;;; **Parameters:**
;;; - s: Input string
;;;
;;; **Returns:** List of words (whitespace-separated substrings)
;;;
;;; **Time Complexity:** O(n) where n is string length
;;;
;;; **Examples:**
;;; - (string-words "hello world test") => ("hello" "world" "test")
;;; - (string-words "  spaced  ") => ("spaced")
;;;
;;; **Notes:** Uses string-split with space delimiter. Trims whitespace first.
(define (string-words s)
  (string-split (string-trim s) " "))

;;; Split string into lines by newline characters.
;;;
;;; **Parameters:**
;;; - s: Input string
;;;
;;; **Returns:** List of lines
;;;
;;; **Time Complexity:** O(n) where n is string length
;;;
;;; **Examples:**
;;; - (string-lines "line1\nline2\nline3") => ("line1" "line2" "line3")
;;; - (string-lines "single") => ("single")
;;;
;;; **Notes:** Uses string-split with newline delimiter.
(define (string-lines s)
  (string-split s "
"))

;; ============================================================================
;; String Padding
;; ============================================================================

;;; Pad string on left with character to reach minimum width.
;;;
;;; **Parameters:**
;;; - s: Input string
;;; - width: Desired minimum width
;;; - char: Single-character string to pad with
;;;
;;; **Returns:** Padded string
;;;
;;; **Time Complexity:** O(n) where n is padding needed
;;;
;;; **Examples:**
;;; - (string-pad-left "42" 5 "0") => "00042"
;;; - (string-pad-left "text" 2 " ") => "text"
;;;
;;; **Notes:** If string is already >= width, returns unchanged.
(define (string-pad-left s width char)
  (let ((len (string-length s)))
    (if (>= len width)
        s
        (string-append (string-repeat char (- width len)) s))))
