import OxFunc.ValueUniverse

namespace OxFunc.PermutationPower

/-- Bounded executable square-and-multiply graph. The exponent is below 2^31,
so 32 steps suffice. Every multiplication binds the observed staged primitive. -/
def powerSteps (mul : Float → Float → Float) : Nat → Nat → Float → Float → Float
  | 0, _, _, acc => acc
  | fuel + 1, exponent, base, acc =>
    if exponent = 0 then acc else
    let acc := if exponent % 2 = 1 then mul acc base else acc
    let exponent := exponent / 2
    powerSteps mul fuel exponent (if exponent = 0 then base else mul base base) acc

def kernel (mul : Float → Float → Float) (decimalPower : Nat → Float)
    (n k : Float) : Except WorksheetErrorCode Float :=
  if !n.isFinite || !k.isFinite || n < 0 || n ≥ 2147483647 ||
      k ≥ 2147483647 || k ≤ -1 then .error .num
  else
    let base := n.floor
    let exponent := k.toUInt64.toNat
    let result := if base == 10 then decimalPower exponent else powerSteps mul 32 exponent base 1
    if result.isFinite then .ok result else .error .num

theorem zero_exponent :
    (kernel (· * ·) (fun _ => 1) 3 0).toOption.map Float.toBits = some (1 : Float).toBits := by native_decide
theorem negative_fractional_exponent_truncates :
    (kernel (· * ·) (fun _ => 1) 3 (-0.75)).toOption.map Float.toBits = some (1 : Float).toBits := by native_decide
theorem negative_fractional_base_rejected :
    (kernel (· * ·) (fun _ => 1) (-0.75) 0).toOption = none := by native_decide

end OxFunc.PermutationPower
