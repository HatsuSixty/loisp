;;; loisp-mode.el --- Major Mode for editing Loisp source code -*- lexical-binding: t -*-

;; Copyright (c) 2022 Roberto Hermenegildo Dias

;; Author: Roberto Hermenegildo Dias
;; URL: https://github.com/robertosixty1/loisp

;; Permission is hereby granted, free of charge, to any person obtaining a copy
;; of this software and associated documentation files (the "Software"), to deal
;; in the Software without restriction, including without limitation the rights
;; to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
;; copies of the Software, and to permit persons to whom the Software is
;; furnished to do so, subject to the following conditions:

;; The above copyright notice and this permission notice shall be included in all
;; copies or substantial portions of the Software.

;; THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
;; IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
;; FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
;; AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
;; LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
;; OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
;; SOFTWARE.

;;; Commentary:
;;
;; Major Mode for editing Loisp source code.

(require 'rx)

(defconst loisp-mode-syntax-table
  (with-syntax-table (copy-syntax-table)
    (syntax-table))
  "Syntax table for `loisp-mode'.")

;; Built-ins are instructions that just generate ir
(defconst loisp-builtins
  '("print" "+" "-" "*" "/" "%" "=" "!=" "<" ">" "<=" ">=" "load64" "store64" "load32" "store32" "load16" "store16" "load8" "store8" "<<" ">>" "&" "|" "!" "castint" "castptr"))

;; Keywords are instructions that do something more than just generating ir
(defconst loisp-keywords
    '("syscall" "setvar" "getvar" "chvar" "while" "if" "block" "ptrto" "alloc" "getmem" "macro" "expand" "pop"))

(defun loisp-wrap-word-rx (s)
  (concat "\\<" s "\\>"))

(defconst loisp-number-rx
  (rx (and
       symbol-start
       (or (and (+ digit) (opt (and (any "eE") (opt (any "-+")) (+ digit))))
           (and "0" (any "xX") (+ hex-digit)))
       (opt (and (any "_" "A-Z" "a-z") (* (any "_" "A-Z" "a-z" "0-9"))))
       symbol-end)))

(defconst loisp-highlights
  `(
    ;; Keywords
    (,(regexp-opt loisp-keywords 'words) . 'font-lock-keyword-face)

    ;; Numbers
    (,(loisp-wrap-word-rx loisp-number-rx) . 'font-lock-constant-face)

    ;; Built-ins
    (,(regexp-opt loisp-builtins 'words) 1 'font-lock-builtin-face)
    ))

;;;###autoload
(define-derived-mode loisp-mode prog-mode "loisp"
  "Major Mode for editing Loisp source code"
  :syntax-table loisp-mode-syntax-table
  (setq font-lock-defaults '(loisp-highlights)))

;;;###autoload
(add-to-list 'auto-mode-alist '("\\.loisp\\'" . loisp-mode))

(provide 'loisp-mode)

;; loisp-mode.el ends here
