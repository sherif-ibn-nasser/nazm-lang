/*! `kust` grammar compiled for Highlight.js 11.10.0 */

(() => {
  const _ident = XRegExp("[_\\p{L}][\\p{L}\\p{N}\\d_]*", "u"),
    ident = XRegExp("\\p{L}[\\p{L}\\p{N}\\d_]*", "u"),
    symbols = /[\s"'+\-*\/%&|~!=#:.<>(){}[\]\\؟،؛]/,
    uni_b_s = `(?<=^|${symbols.source})`, // Start of a unicode word boundary
    uni_b_e = `(?=$|${symbols.source})`, // End of a unicode word boundary
    num_suffix = "([صط](1|2|4|8)?|ع(4|8))?" + uni_b_e,
    escaped_char = /\\([\\0"'خفسرصج$]|ي[0-9a-fA-F]{4})/,
    keywords = [
      "تصدير",
      "تخصيص",
      "هيكل",
      "دالة",
      "احجز",
      "متغير",
      "ثابت",
      "تشغيل",
      "لو",
      "وإلا",
      "عندما",
      "تكرار",
      "طالما",
      "وصل",
      "قطع",
      "أرجع",
      "صحيح",
      "فاسد",
      "على",
    ],
    non_keywords = XRegExp(
      "(?!" + `(?:${keywords.join("|")})` + uni_b_e + ")",
      "u"
    ),
    invoke = XRegExp.union(
      [XRegExp(uni_b_s), non_keywords, _ident, XRegExp(uni_b_e), /(?=\s*[{(])/],
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
          literal: ["صحيح", "فاسد"],
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
            "شرط",
            "نص",
          ],
        },
        illegal: "</",
        contains: [
          e.C_LINE_COMMENT_MODE,
          e.COMMENT("/\\*", "\\*/"),
          {
            scope: "string",
            begin: '"',
            end: '"',
            illegal: ["\\n"],
            contains: [
              {
                scope: "char.escape",
                match: escaped_char,
              },
            ],
          },
          {
            scope: "string",
            variants: [
              {
                begin: [/'/, escaped_char, /'/],
                beginScope: { 2: "char.escape" },
              },
              {
                begin: /'.'/,
              },
            ],
          },
          {
            scope: "number",
            variants: [
              {
                begin: uni_b_s + "2#([01,]+)" + num_suffix,
              },
              { begin: uni_b_s + "8#([0-7,]+)" + num_suffix },
              {
                begin: uni_b_s + "16#([A-Fa-f0-9,]+)" + num_suffix,
              },
              {
                begin:
                  uni_b_s +
                  "(\\d[\\d,]*(\\.[0-9,]+)?(\\^\\^[+-]?[0-9,]+)?)" +
                  num_suffix,
              },
            ],
            relevance: 0,
          },
          {
            begin: [/هيكل/, /\s+/, _ident],
            beginScope: { 1: "keyword", 3: "title.class" },
          },
          {
            begin: [/دالة/, /\s+/, _ident],
            beginScope: { 1: "keyword", 3: "title.function" },
          },
          {
            begin: [/احجز/, /\s+/, /(?:متغير\s+)?/, _ident],
            beginScope: { 1: "keyword", 3: "keyword", 4: "variable" },
          },
          {
            scope: "title.function.invoke",
            relevance: 0,
            begin: invoke,
          },
        ],
      };
    };
  })();

  hljs.registerLanguage("nazm", e);
})();
