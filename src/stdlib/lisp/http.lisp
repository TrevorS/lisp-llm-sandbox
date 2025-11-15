;; ABOUTME: HTTP utilities - High-level wrappers around http-request builtin
;; This library provides convenience functions for common HTTP patterns

;; ============================================================================
;; HTTP Status Checking
;; ============================================================================

;;; Check if HTTP response status indicates success.
;;;
;;; **Parameters:**
;;; - response: Response map from http-request
;;;
;;; **Returns:** #t if status is 2xx (200-299), #f otherwise
;;;
;;; **Time Complexity:** O(1)
;;;
;;; **Examples:**
;;; - (let ((resp (http-request "https://example.com" {:method "GET"}))) (http:check-status resp))
;;; - (if (http:check-status resp) (process-data resp) (handle-error resp))
;;;
;;; **Notes:** Helper predicate for checking HTTP success codes.
;;; Returns #f for non-response maps (network errors, etc).
(define (http:check-status response)
  (if (map? response)
      (let ((status (map-get response :status)))
        (if (number? status)
            (and (>= status 200) (< status 300))
            #f))
      #f))

;;; Get response body as string.
;;;
;;; **Parameters:**
;;; - response: Response map from http-request
;;;
;;; **Returns:** Response body string, or empty string if not found
;;;
;;; **Examples:**
;;; - (http:body (http-request "https://example.com" {:method "GET"}))
;;;
;;; **Notes:** Safe accessor that returns empty string if response is invalid.
(define (http:body response)
  (if (map? response)
      (map-get response :body "")
      ""))

;;; Get response status code as number.
;;;
;;; **Parameters:**
;;; - response: Response map from http-request
;;;
;;; **Returns:** HTTP status code (e.g., 200, 404), or 0 if not found
;;;
;;; **Examples:**
;;; - (http:status (http-request "https://example.com" {:method "GET"}))
;;;
;;; **Notes:** Safe accessor that returns 0 if response is invalid.
(define (http:status response)
  (if (map? response)
      (map-get response :status 0)
      0))
