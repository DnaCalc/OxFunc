#include <windows.h>
#include <cwchar>
#include <vector>

#include "xlcall.h"

struct OxFuncShimArg {
    int tag;
    double number;
    int logical;
    const wchar_t* text_ptr;
    unsigned int text_len;
    int error_code;
};

struct OxFuncShimResult {
    int tag;
    double number;
    int error_code;
};

enum {
    kArgTagMissing = 0,
    kArgTagEmpty = 1,
    kArgTagNumber = 2,
    kArgTagText = 3,
    kArgTagLogical = 4,
    kArgTagError = 5,
};

enum {
    kResultTagNumber = 1,
    kResultTagError = 2,
};

extern "C" int __stdcall oxfunc_abs_eval_shim(const OxFuncShimArg* arg, OxFuncShimResult* out);

struct RegSpec {
    const wchar_t* export_name;
    const wchar_t* type_text;
    const wchar_t* function_name;
    const wchar_t* arg_names;
};

static const RegSpec kSpecs[] = {
    {L"OX_ABS", L"QU", L"ox_ABS", L"value"},
    {L"OX_ABS_Q", L"BB", L"ox_ABS_Q", L"value"},
    {L"OX_PI", L"B", L"ox_PI", L""},
};

static std::vector<XCHAR> ToXlString(const wchar_t* src) {
    size_t len = std::wcslen(src);
    if (len > 32767) {
        len = 32767;
    }
    std::vector<XCHAR> out(len + 1);
    out[0] = static_cast<XCHAR>(len);
    for (size_t i = 0; i < len; ++i) {
        out[i + 1] = static_cast<XCHAR>(src[i]);
    }
    return out;
}

static XLOPER12 MakeStr(XCHAR* s) {
    XLOPER12 x = {};
    x.xltype = xltypeStr;
    x.val.str = s;
    return x;
}

static XLOPER12 MakeInt(int v) {
    XLOPER12 x = {};
    x.xltype = xltypeInt;
    x.val.w = v;
    return x;
}

static LPXLOPER12 MakeResultError(int error_code) {
    static thread_local XLOPER12 result;
    result = {};
    result.xltype = xltypeErr;
    result.val.err = error_code;
    return &result;
}

static LPXLOPER12 MakeResultNumber(double value) {
    static thread_local XLOPER12 result;
    result = {};
    result.xltype = xltypeNum;
    result.val.num = value;
    return &result;
}

static LPXLOPER12 MakeResultMulti(unsigned int rows, unsigned int cols, const std::vector<XLOPER12>& items) {
    const size_t count = static_cast<size_t>(rows) * static_cast<size_t>(cols);
    auto* result = new XLOPER12{};
    auto* storage = new XLOPER12[count]{};
    for (size_t i = 0; i < count; ++i) {
        storage[i] = items[i];
    }
    result->xltype = xltypeMulti | xlbitDLLFree;
    result->val.array.rows = rows;
    result->val.array.columns = cols;
    result->val.array.lparray = storage;
    return result;
}

static bool CoerceReferenceToValue(LPXLOPER12 arg, XLOPER12* coerced_out) {
    XLOPER12 coerced = {};
    LPXLOPER12 args[1] = {arg};
    int ret = Excel12v(xlCoerce, &coerced, 1, args);
    if (ret != xlretSuccess) {
        return false;
    }
    *coerced_out = coerced;
    return true;
}

static const DWORD kTypeMask = 0x0FFF;

static bool ToShimArg(LPXLOPER12 arg, OxFuncShimArg* out, bool* used_temp, XLOPER12* temp) {
    if (arg == nullptr) {
        out->tag = kArgTagMissing;
        *used_temp = false;
        return true;
    }

    LPXLOPER12 value = arg;
    DWORD ty = value->xltype & kTypeMask;
    if (ty == xltypeRef || ty == xltypeSRef) {
        if (!CoerceReferenceToValue(value, temp)) {
            return false;
        }
        value = temp;
        ty = value->xltype & kTypeMask;
        *used_temp = true;
    } else {
        *used_temp = false;
    }

    out->number = 0.0;
    out->logical = 0;
    out->text_ptr = nullptr;
    out->text_len = 0;
    out->error_code = xlerrValue;

    switch (ty) {
        case xltypeMissing:
            out->tag = kArgTagMissing;
            return true;
        case xltypeNil:
            out->tag = kArgTagEmpty;
            return true;
        case xltypeNum:
            out->tag = kArgTagNumber;
            out->number = value->val.num;
            return true;
        case xltypeBool:
            out->tag = kArgTagLogical;
            out->logical = value->val.xbool ? 1 : 0;
            return true;
        case xltypeErr:
            out->tag = kArgTagError;
            out->error_code = value->val.err;
            return true;
        case xltypeStr: {
            out->tag = kArgTagText;
            if (value->val.str == nullptr) {
                out->text_ptr = nullptr;
                out->text_len = 0;
                return true;
            }
            const XCHAR* pstr = value->val.str;
            unsigned int len = static_cast<unsigned int>(pstr[0]);
            out->text_ptr = reinterpret_cast<const wchar_t*>(pstr + 1);
            out->text_len = len;
            return true;
        }
        default:
            out->tag = kArgTagError;
            out->error_code = xlerrValue;
            return true;
    }
}

static bool EvalAbsScalarFromXloper(LPXLOPER12 arg, OxFuncShimResult* shim_result) {
    OxFuncShimArg shim_arg = {};
    bool used_temp = false;
    XLOPER12 temp = {};
    if (!ToShimArg(arg, &shim_arg, &used_temp, &temp)) {
        return false;
    }
    int ok = oxfunc_abs_eval_shim(&shim_arg, shim_result);
    if (used_temp) {
        Excel12(xlFree, nullptr, 1, &temp);
    }
    return ok == 1;
}

extern "C" __declspec(dllexport) LPXLOPER12 __stdcall OX_ABS(LPXLOPER12 arg) {
    LPXLOPER12 value = arg;
    bool used_temp = false;
    XLOPER12 temp = {};

    if (value != nullptr) {
        DWORD ty = value->xltype & kTypeMask;
        if (ty == xltypeRef || ty == xltypeSRef) {
            if (!CoerceReferenceToValue(value, &temp)) {
                return MakeResultError(xlerrValue);
            }
            value = &temp;
            used_temp = true;
        }
    }

    DWORD ty = (value == nullptr) ? xltypeMissing : (value->xltype & kTypeMask);
    if (ty == xltypeMulti) {
        const auto rows = value->val.array.rows;
        const auto cols = value->val.array.columns;
        const auto count = static_cast<size_t>(rows) * static_cast<size_t>(cols);
        std::vector<XLOPER12> mapped(count);
        LPXLOPER12 items = value->val.array.lparray;

        for (size_t i = 0; i < count; ++i) {
            OxFuncShimResult shim_result = {};
            if (!EvalAbsScalarFromXloper(&items[i], &shim_result)) {
                mapped[i] = {};
                mapped[i].xltype = xltypeErr;
                mapped[i].val.err = xlerrValue;
                continue;
            }
            mapped[i] = {};
            if (shim_result.tag == kResultTagNumber) {
                mapped[i].xltype = xltypeNum;
                mapped[i].val.num = shim_result.number;
            } else {
                mapped[i].xltype = xltypeErr;
                mapped[i].val.err = shim_result.error_code;
            }
        }

        if (used_temp) {
            Excel12(xlFree, nullptr, 1, &temp);
        }
        return MakeResultMulti(rows, cols, mapped);
    }

    OxFuncShimResult shim_result = {};
    if (!EvalAbsScalarFromXloper(value, &shim_result)) {
        if (used_temp) {
            Excel12(xlFree, nullptr, 1, &temp);
        }
        return MakeResultError(xlerrValue);
    }
    if (used_temp) {
        Excel12(xlFree, nullptr, 1, &temp);
    }
    if (shim_result.tag == kResultTagNumber) {
        return MakeResultNumber(shim_result.number);
    }
    if (shim_result.tag == kResultTagError) {
        return MakeResultError(shim_result.error_code);
    }
    return MakeResultError(xlerrValue);
}

extern "C" __declspec(dllexport) void __stdcall xlAutoFree12(LPXLOPER12 to_free) {
    if (to_free == nullptr) {
        return;
    }
    if ((to_free->xltype & kTypeMask) == xltypeMulti) {
        delete[] to_free->val.array.lparray;
    }
    delete to_free;
}

static int RegisterOne(const wchar_t* module_path, const RegSpec& spec) {
    auto dll = ToXlString(module_path);
    auto proc = ToXlString(spec.export_name);
    auto type = ToXlString(spec.type_text);
    auto name = ToXlString(spec.function_name);
    auto args = ToXlString(spec.arg_names);
    auto cat = ToXlString(L"OxFunc Bridge");

    XLOPER12 xDll = MakeStr(dll.data());
    XLOPER12 xProc = MakeStr(proc.data());
    XLOPER12 xType = MakeStr(type.data());
    XLOPER12 xName = MakeStr(name.data());
    XLOPER12 xArgs = MakeStr(args.data());
    XLOPER12 xMacro = MakeInt(1);
    XLOPER12 xCat = MakeStr(cat.data());
    XLOPER12 xRegId = {};

    LPXLOPER12 opers[7] = {&xDll, &xProc, &xType, &xName, &xArgs, &xMacro, &xCat};
    int ret = Excel12v(xlfRegister, &xRegId, 7, opers);
    return (ret == xlretSuccess) ? 1 : 0;
}

extern "C" __declspec(dllexport) int __stdcall oxfunc_register_all(const wchar_t* module_path) {
    if (module_path == nullptr || module_path[0] == L'\0') {
        return 0;
    }
    for (const auto& spec : kSpecs) {
        if (!RegisterOne(module_path, spec)) {
            return 0;
        }
    }
    return 1;
}
