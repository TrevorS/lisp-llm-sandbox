;; ABOUTME: Testing framework - Assertion macros and test result formatting
;; This library provides test definition and reporting utilities

;; ============================================================================
;; Test Registration
;; ============================================================================

;;; Define a test and register it in the test registry.
;;;
;;; **Usage:** (define-test test-name body-expr)
;;;
;;; **Parameters:**
;;; - name: String name for the test
;;; - body: Single expression to execute (use begin for multiple)
;;;
;;; **Examples:**
;;; - (define-test math-test (assert-equal (+ 2 2) 4))
;;; - (define-test string-test (begin (assert-equal 1 1) (assert-equal 2 2)))
;;;
;;; **Notes:** Macro expands to (register-test name (lambda () body))
(defmacro define-test (name body)
  `(register-test ,name (lambda () ,body)))

;; ============================================================================
;; Test Results Formatting
;; ============================================================================

;;; Get value from map by key.
;;;
;;; **Parameters:**
;;; - m: Map to query
;;; - key: Keyword key to look up
;;; - default: Value to return if key not found
;;;
;;; **Returns:** Value at key, or default if not found
;;;
;;; **Examples:**
;;; - (map-get {:a 1 :b 2} :a) => 1
;;; - (map-get {:a 1} :c nil) => nil
;;;
;;; **Notes:** This is a builtin primitive, included here for reference.
;;; In updated code, use it instead of alist-get.

;;; Print formatted test results from run-all-tests.
;;;
;;; **Parameters:**
;;; - results: Result map from run-all-tests with :passed, :failed, :total, :tests
;;;
;;; **Returns:** nil (prints to console)
;;;
;;; **Examples:**
;;; - (print-test-summary (run-all-tests))
;;;
;;; **Notes:** Results map structure: {:passed N :failed M :total T :tests [...]})
(define (print-test-summary results)
  (let ((passed (map-get results :passed 0))
        (failed (map-get results :failed 0))
        (total (map-get results :total 0))
        (tests (map-get results :tests '())))
    (begin
      (println "")
      (println "=================================")
      (println "        TEST RESULTS")
      (println "=================================")
      (print-test-details tests)
      (println "")
      (println (string-append "Total:  " (number->string total)))
      (println (string-append "Passed: " (number->string passed)))
      (println (string-append "Failed: " (number->string failed)))
      (if (= failed 0)
          (println "\nAll tests passed!")
          (println (string-append "\n" (number->string failed) " test(s) failed")))
      (println "=================================")
      nil)))

;;; Print each test result with status.
;;;
;;; **Parameters:**
;;; - tests: List of test result maps with :name, :status, :message keys
;;;
;;; **Returns:** nil (prints to console)
;;;
;;; **Examples:**
;;; - (print-test-details (map-get (run-all-tests) :tests))
;;;
;;; **Notes:** Each test result is a map: {:name "test-name" :status 'passed :message ""}
(define (print-test-details tests)
  (if (empty? tests)
      nil
      (begin
        (let ((test-result (car tests)))
          (let ((name (map-get test-result :name ""))
                (status (map-get test-result :status 'unknown))
                (msg (map-get test-result :message "")))
            (begin
              (print (if (= status 'passed) "  PASS: " "  FAIL: "))
              (println name)
              (if (= status 'passed)
                  nil
                  (println (string-append "        " msg))))))
        (print-test-details (cdr tests)))))
