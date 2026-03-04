import Lake
open Lake DSL

package «oxfunc_formal» where

lean_lib OxFunc where

@[default_target]
lean_exe «oxfunc_formal_check» where
  root := `Main

