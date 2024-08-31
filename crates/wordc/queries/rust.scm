; Identifiers to "case-split"
[
  (identifier)
  (field_identifier)
  (type_identifier)
  (primitive_type)
] @ident


; String literals' content to process as a sentence
(string_content) @sentence.string

; Comments' content to process as a sentence
(doc_comment) @sentence.comment
; (RAW) Exclude rust-specific comment patterns
[
  (line_comment !doc)
  (block_comment !doc)
] @sentence.comment.raw
