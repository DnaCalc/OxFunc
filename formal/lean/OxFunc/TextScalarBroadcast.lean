import OxFunc.CoercionPrimitives
import OxFunc.NumericTextRendering

namespace OxFunc

-- Prepared rectangular arguments only. Reference resolution and scalar/1x1
-- publication belong to the existing adapter/evaluator boundary.
inductive TextBroadcastArgument where
  | scalar (value : CoercionInput)
  | rectangular (rows : List (List CoercionInput))
  deriving Repr, DecidableEq

def textArgumentShape : TextBroadcastArgument → Nat × Nat
  | .scalar _ => (1, 1)
  | .rectangular rows => (rows.length, (rows.headD []).length)

def textArgumentAt (arg : TextBroadcastArgument) (row col : Nat) : CoercionInput :=
  match arg with
  | .scalar value => value
  | .rectangular rows =>
    let shape := textArgumentShape arg
    let r := if shape.1 = 1 then 0 else row
    let c := if shape.2 = 1 then 0 else col
    ((rows[r]?).bind (fun values => values[c]?)).getD (.error .na)

def textBroadcastGrid (scalar : List CoercionInput → CoercionInput)
    (args : List TextBroadcastArgument) : List (List CoercionInput) :=
  let shape := args.foldl (fun s a =>
    let t := textArgumentShape a
    (max s.1 t.1, max s.2 t.2)) (1, 1)
  (List.range shape.1).map (fun r => (List.range shape.2).map (fun c =>
    scalar (args.map (fun a => textArgumentAt a r c))))

-- A padded coordinate participates as an ordinary #N/A argument. It does not
-- replace an earlier scalar argument's error before the function sees it.
def textFirstError (args : List CoercionInput) : CoercionInput :=
  (args.find? (fun x => match x with | .error _ => true | _ => false)).getD (.text "")

theorem text_padding_preserves_argument_error_order :
    textBroadcastGrid textFirstError
      [.rectangular [[.error .div0, .error .ref], [.text "", .error .num]],
       .rectangular [[.number 1], [.number 2], [.number 3]]]
    = [[.error .div0, .error .ref], [.text "", .error .num], [.error .na, .error .na]] := by
  native_decide

end OxFunc
