# orgmd_parser
org-mode and markdown parser for logorg.

## WebAssembly Exported Methods

- For javascript
  - parse_markdown( string )

- For other envs (Low-Level API)
  - allocate( size ) -> pointer
  - deallocate( pointer, capacity )
  - deallocate_str( string_pointer )
  - ffi_parse_markdown( string_pointer ) -> string_pointer

