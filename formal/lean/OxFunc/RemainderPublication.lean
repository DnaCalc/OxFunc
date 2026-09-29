import OxFunc.NumericPublication

namespace OxFunc.RemainderPublication

/-- Executable MOD publication graph for retained normal binary64 inputs.
The fmod primitive is supplied explicitly. Tiny branch identity is empirical;
this is not a proof about the implementation of the Excel primitive. -/
def modWithPrimitive (fmod : Float → Float → Float) (number divisor : Float) :
    Except WorksheetErrorCode Float :=
  if divisor == 0 then .error .div0
  else if (number / divisor).abs ≥ 1125900000000 then .error .num
  else
    let remainder := fmod number divisor
    let tiny := remainder != 0 && remainder.abs < Float.ofBits 0x0010000000000000
    let powerOfTwo := divisor.toBits &&& 0x000fffffffffffff == 0
    if tiny && (!powerOfTwo || (number / divisor).abs ≥ 67108864) then .error .num
    else
      let adjusted := if remainder != 0 && ((remainder < 0) != (divisor < 0))
        then remainder + divisor else remainder
      let endpoint := if adjusted == 0 || adjusted == divisor then 0 else adjusted
      let result := if remainder != 0 && remainder.abs < Float.ofBits 0x0001000000000000
          && ((number < 0) != (divisor < 0))
        then divisor - remainder else endpoint
      if result != 0 && result.abs < Float.ofBits 0x0010000000000000
      then .error .num else .ok result

theorem mod_normalizes_negative_zero :
    ((modWithPrimitive (fun _ _ => Float.ofBits 0x8000000000000000)
      (-2) 1).toOption.map Float.toBits) = some 0 := by native_decide

theorem mod_normalizes_divisor_endpoint :
    ((modWithPrimitive (fun _ _ => Float.ofBits 0x0010000000000000)
      (Float.ofBits 0x0010000000000000) (-1)).toOption.map Float.toBits) = some 0 := by native_decide

theorem mod_strict_below :
    ((modWithPrimitive (fun _ _ => Float.ofBits 0x8000400000000000)
      (Float.ofBits 0x8010400000000000) (Float.ofBits 0x0010000000000000)).toOption.map
      Float.toBits) = some 0x0010400000000000 := by native_decide

theorem mod_strict_at :
    ((modWithPrimitive (fun _ _ => Float.ofBits 0x8001000000000000)
      (Float.ofBits 0x8011000000000000) (Float.ofBits 0x0010000000000000)).toOption.map
      Float.toBits) = none := by native_decide

theorem mod_large_quotient_tiny :
    ((modWithPrimitive (fun _ _ => Float.ofBits 0x8008000000000000)
      (Float.ofBits 0x81b0000002000000) (Float.ofBits 0x0010000000000000)).toOption.map
      Float.toBits) = none := by native_decide

end OxFunc.RemainderPublication
