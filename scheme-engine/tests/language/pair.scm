;; ========
;; Pairs
;; ========

(define a (cons 1 2))
(assert (pair? a))
(assert (pair? (cons 1 2)))
(assert (not (pair? 1)))

(define b (list 1 2 3))
(assert (list? b))
(assert (list? (list 1 2 3)))
(assert (not (list? (cons 1 2)))) ;; no nil tail
