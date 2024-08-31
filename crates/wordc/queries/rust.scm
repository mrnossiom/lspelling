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
  (comment)
] @sentence.comment
