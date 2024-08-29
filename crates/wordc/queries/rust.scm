; Process as is
[
  (type_identifier)
  (primitive_type)
  (field_identifier)
  (identifier)
] @ident


; Exclude quotes from the sides
[
  (string_literal)
  (raw_string_literal)
] @sentence.string

; Exclude comment pattern
[
  (line_comment)
  (block_comment)
] @sentence.comment
