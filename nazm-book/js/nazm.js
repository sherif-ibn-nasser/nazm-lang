/*! `kust` grammar compiled for Highlight.js 11.10.0 */

(() => {
  const _ident = XRegExp("[_\\p{L}][\\p{L}\\p{N}\\d_]*", "u"),
    ident = XRegExp("\\p{L}[\\p{L}\\p{N}\\d_]*", "u"),
    symbols = /[\s"'+\-*\/%&|~!=#:.<>(){}[\]\\؟،؛]/,
    uni_b_s = `(?<=^|${symbols.source})`, // Start of a unicode word boundary
    uni_b_e = `(?=$|${symbols.source})`, // End of a unicode word boundary
    num_suffix = "([صط](1|2|4|8)?|ع(4|8))?" + uni_b_e,
    keywords = [
      "تصدير",
      "تخصيص",
      "تصنيف",
      "دالة",
      "احجز",
      "متغير",
      "ثابت",
      "تشغيل",
      "لو",
      "وإلا",
      "تكرار",
      "طالما",
      "وصل",
      "قطع",
      "أرجع",
      "مؤكد",
      "محال",
    ],
    non_keywords = XRegExp(
      "(?!" + `(?:${keywords.join("|")})` + uni_b_e + ")",
      "u"
    ),
    invoke = XRegExp.union(
      [XRegExp(uni_b_s), non_keywords, _ident, XRegExp(uni_b_e), /(?=\s*\()/],
      "u",
      {
        conjunction: "none",
      }
    );

  var e = (() => {
    "use strict";

    return (e) => {
      return {
        name: "nazm",
        aliases: ["نظم", "nazm"],
        keywords: {
          $pattern: ident,
          keyword: keywords,
          literal: ["مؤكد", "محال"],
          type: [
            "1ص",
            "ص2",
            "ص4",
            "ص8",
            "ص",
            "ط1",
            "ط2",
            "ط4",
            "ط8",
            "ط",
            "ع4",
            "ع8",
            "متن",
            "حرف",
            "خبر",
            "نص",
          ],
        },
        illegal: "</",
        contains: [
          e.C_LINE_COMMENT_MODE,
          e.COMMENT("/\\*", "\\*/"),
          e.inherit(e.QUOTE_STRING_MODE, { begin: /b?"/, illegal: null }),
          {
            className: "string",
            variants: [
              { begin: /b?r(#*)"(.|\n)*?"\1(?!#)/ },
              {
                begin: /b?'\\?(x\w{2}|u\w{4}|U\w{8}|.)'/,
              },
            ],
          },
          {
            className: "number",
            variants: [
              {
                begin: uni_b_s + "1#([01_]+)" + num_suffix,
              },
              { begin: uni_b_s + "8#([0-7_]+)" + num_suffix },
              { begin: uni_b_s + "10#([0-7_]+)" + num_suffix },
              {
                begin: uni_b_s + "16#([A-Fa-f0-9_]+)" + num_suffix,
              },
              {
                begin:
                  uni_b_s +
                  "(\\d[\\d_]*(\\.[0-9_]+)?(\\^\\^[+-]?[0-9_]+)?)" +
                  num_suffix,
              },
            ],
            relevance: 0,
          },
          {
            begin: [/تصنيف/, /\s+/, _ident],
            className: { 1: "keyword", 3: "title.class" },
          },
          {
            begin: [/دالة/, /\s+/, _ident],
            className: { 1: "keyword", 3: "title.function" },
          },
          {
            begin: [/احجز/, /\s+/, /(?:متغير\s+)?/, _ident],
            className: { 1: "keyword", 3: "keyword", 4: "variable" },
          },
          {
            className: "title.function.invoke",
            relevance: 0,
            begin: invoke,
          },
          // { className: "symbol", begin: symbols },
        ],
      };
    };
  })();

  hljs.registerLanguage("nazm", e);
})();
