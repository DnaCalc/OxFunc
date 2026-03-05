#include <windows.h>
#include <cwchar>
#include <vector>

#include "xlcall.h"

struct RegSpec {
    const wchar_t* export_name;
    const wchar_t* type_text;
    const wchar_t* function_name;
    const wchar_t* arg_names;
};

static const RegSpec kSpecs[] = {
    {L"OXFP_NEG_ZERO", L"B", L"OXFP_NEG_ZERO", L""},
    {L"OXFP_POS_INF", L"B", L"OXFP_POS_INF", L""},
    {L"OXFP_NEG_INF", L"B", L"OXFP_NEG_INF", L""},
    {L"OXFP_QNAN", L"BB", L"OXFP_QNAN", L"payload"},
    {L"OXFP_SNAN", L"BB", L"OXFP_SNAN", L"payload"},
    {L"OXFP_BITS_ECHO", L"BB", L"OXFP_BITS_ECHO", L"value"},
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

static int RegisterOne(const wchar_t* module_path, const RegSpec& spec) {
    auto dll = ToXlString(module_path);
    auto proc = ToXlString(spec.export_name);
    auto type = ToXlString(spec.type_text);
    auto name = ToXlString(spec.function_name);
    auto args = ToXlString(spec.arg_names);
    auto cat = ToXlString(L"OxFunc FP Probe");

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

extern "C" __declspec(dllexport) int __stdcall oxfp_register_all(const wchar_t* module_path) {
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
