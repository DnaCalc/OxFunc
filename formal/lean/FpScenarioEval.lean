import Std

open Std

def pow10 (n : Nat) : Float :=
  Id.run do
    let mut v : Float := 1.0
    for _i in [0:n] do
      v := v * 10.0
    return v

def scenarioValue (scenarioId : String) : Option (Float × String × String) :=
  let tenPow307 := pow10 307
  let tenPow20 := pow10 20
  let oneE307 := tenPow307
  let oneENeg307 := 1.0 / tenPow307
  match scenarioId with
  | "FP2-001" => some (-0.0, "=-0", "signed zero literal")
  | "FP2-002" => some (0.0 * (-1.0), "=0*-1", "signed zero arithmetic")
  | "FP2-003" => some ((-oneENeg307) / tenPow20, "=-1E-307/1E20", "underflow candidate")
  | "FP2-004" => some ((oneE307) * 10.0, "=(10^307)*10", "overflow candidate")
  | "FP2-005" => some (0.0 / 0.0, "=0/0", "invalid operation")
  | "FP2-006" => some (1.0 / 0.0, "=1/0", "division by zero +")
  | "FP2-007" => some ((-1.0) / 0.0, "=-1/0", "division by zero -")
  | "FP2-008" => some (oneENeg307 / 1000.0, "=1E-307/1000", "subnormal candidate +")
  | "FP2-009" => some ((-oneENeg307) / 1000.0, "=-1E-307/1000", "subnormal candidate -")
  | "FP2-010" => some (oneENeg307 / 1000.0, "=VALUE(\"1E-307\")/1000", "text coercion approximated as parse-equivalent")
  | _ => none

def classifyFloat (x : Float) : String :=
  if x.isNaN then
    "value:nan"
  else if x.isInf then
    if x < 0.0 then "value:-inf" else "value:+inf"
  else if x == 0.0 then
    if Float.toBits x == Float.toBits (-0.0) then "value:-0" else "value:+0"
  else
    "value:finite"

def sanitize (s : String) : String :=
  (s.replace "|" "/").replace "\n" " "

def emitOk (scenarioId : String) (obsClass : String) (x : Float) (formulaHint : String) (notes : String) : IO Unit := do
  let text := sanitize (toString x)
  let bits := toString (Float.toBits x)
  let formula := sanitize formulaHint
  let noteText := sanitize notes
  IO.println s!"ok|{scenarioId}|{obsClass}|{text}|{bits}|{formula}|{noteText}"

def emitErr (scenarioId : String) (msg : String) : IO Unit := do
  IO.println s!"err|{scenarioId}|{sanitize msg}"

def evalAndEmit (scenarioId : String) : IO Unit := do
  if scenarioId.isEmpty then
    emitErr scenarioId "missing scenario id"
  else
    match scenarioValue scenarioId with
    | none => emitErr scenarioId s!"unsupported scenario: {scenarioId}"
    | some (x, formulaHint, notes) =>
        emitOk scenarioId (classifyFloat x) x formulaHint notes

def main (args : List String) : IO Unit := do
  if args.isEmpty then
    emitErr "" "missing scenario id"
  else
    for scenarioId in args do
      evalAndEmit scenarioId
