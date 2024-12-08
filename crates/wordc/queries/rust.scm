; Identifiers to "case-split"
[
  (identifier)
  (field_identifier)
  (type_identifier)
] @ident


; String literals' content to process as a sentence
(string_content) @sentence.string

; Comments' content to process as a sentence
[
  (doc_comment)
  ; TODO: needs custom handling to strip slashes either with a Rust TS grammar
  ; that supports comments content, or with an `offset!` meta node handled
  ; at runtime
  ; (comment)
] @sentence.comment
