# W46 Runtime Requirements

1. Host/OxFml owns raw Excel C API dispatch, DLL/code-resource loading, and actual external invocation.
2. OxFunc owns:
   - `REGISTER.ID` request normalization
   - `CALL` target normalization
   - register-id lookup vs direct-registration resolution split
   - projected worksheet value/error result
3. The current admitted baseline is the explicit-`type_text` Win32 lane:
   - `Kernel32/GetTickCount`
   - `Kernel32/MulDiv`
4. The current admitted baseline also includes the seeded zero-argument omitted-`type_text` lane:
   - `REGISTER.ID("Kernel32","GetTickCount")`
   - `CALL("Kernel32","GetTickCount")`
5. Broader argument-bearing omitted-`type_text` behavior is not yet pinned.
6. Current typed OxFunc seam is:
   - `RegisterIdRequest`
   - `RegisteredExternalDescriptor`
   - `RegisteredExternalCallRequest`
   - `RegisteredExternalProvider`
