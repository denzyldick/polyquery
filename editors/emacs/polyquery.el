;;; polyquery.el --- SQL detection, validation, and execution via LSP -*- lexical-binding: t; -*-

;; Copyright (C) 2026 Polyquery Contributors

;; Author: Polyquery <https://github.com/anomalyco/polyquery>
;; Version: 0.1.0
;; Package-Requires: ((emacs "29.1") (eglot "1.0"))
;; Keywords: languages, sql, tools
;; URL: https://github.com/anomalyco/polyquery

;;; Commentary:
;; Provides LSP-based SQL detection, validation, autocomplete,
;; and execution across 15+ programming languages.

;;; Code:

(require 'eglot)

(defgroup polyquery nil
  "Polyquery: SQL-aware LSP for any language."
  :group 'tools)

(defcustom polyquery-server-path "polyquery"
  "Path to the polyquery LSP server binary."
  :type 'string
  :group 'polyquery)

(defcustom polyquery-database-url nil
  "Database URL for schema introspection and query execution.
Example: postgresql://user:pass@localhost:5432/mydb"
  :type '(choice (const :tag "Not set" nil) string)
  :group 'polyquery)

(defvar polyquery--results-buffer "*polyquery-results*"
  "Buffer name for query results.")

;;;###autoload
(defun polyquery-set-database-url (url)
  "Set the database URL for Polyquery.
URL should be a connection string like postgresql://user:pass@host/db."
  (interactive "sDatabase URL: ")
  (setq polyquery-database-url url)
  (when (and (eglot-current-server)
             (jsonrpc--running-p (eglot-current-server)))
    (eglot-reconnect (eglot-current-server)))
  (message "Polyquery database URL set"))

;;;###autoload
(defun polyquery-clear-database-url ()
  "Clear the stored database URL."
  (interactive)
  (setq polyquery-database-url nil)
  (message "Polyquery database URL cleared"))

;;;###autoload
(defun polyquery-run-query (&optional sql)
  "Execute SQL query via Polyquery.
If SQL is nil, uses the region or current line."
  (interactive)
  (unless sql
    (if (use-region-p)
        (setq sql (buffer-substring-no-properties
                   (region-beginning) (region-end)))
      (setq sql (thing-at-point 'line t))))
  (unless (and sql (not (string-blank-p sql)))
    (user-error "No SQL to execute"))

  (let ((server (eglot-current-server)))
    (unless server
      (user-error "Polyquery LSP server not running"))
    (jsonrpc-request
     server
     :workspace/executeCommand
     `(:command "polyquery.runQuery"
                :arguments [, (buffer-file-name) ,sql])
     :timeout 30
     :success (cl-function
               (lambda (&key type text message &allow-other-keys)
                 (cond
                  ((equal type "error")
                   (user-error "Query error: %s" message))
                  (text
                   (with-current-buffer (get-buffer-create polyquery--results-buffer)
                     (read-only-mode -1)
                     (erase-buffer)
                     (insert text)
                     (goto-char (point-min))
                     (read-only-mode 1)
                     (display-buffer (current-buffer))))))))))

;;;###autoload
(defun polyquery-eglot-contact ()
  "Return the Eglot contact for Polyquery."
  `(,polyquery-server-path
    :initializationOptions
    ,(when polyquery-database-url
       `(:POLYQUERY_DATABASE_URL ,polyquery-database-url))))

;;;###autoload
(add-to-list 'eglot-server-programs
             '((js-mode typescript-mode python-mode go-mode ruby-mode
                        java-mode rust-mode php-mode csharp-mode c-mode c++-mode
                        kotlin-mode scala-mode swift-mode elixir-mode)
                . (polyquery-eglot-contact)))

(provide 'polyquery)

;;; polyquery.el ends here
