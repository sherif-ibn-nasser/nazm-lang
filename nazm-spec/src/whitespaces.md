## المسافات

المسافات تشمل الفراغات والأسطر الجديدة. في لغة **نظم**، يتم تجاهل المسافات إلا إذا كانت داخل [القيم النصية](chars_and_strings.md). إليك المسافات التي يتم تجاهلها في الكود، وترميز اليونيكود الخاص لها:

| ترميز يونيكود | الحرف في نظم | الحرف في C |  وظيفة الحرف   | 
| :----------: | :--------: | :------------: | :-----------: |
|     0009     |     `\ف`     |    `\t`    |  مسافة أفقية    |
|     000A|     `\س`     |    `\n`    |    سطر جديد          |
|     000B |     `\ر`     |    `\v`    |  مسافة رأسية        |
|     000C|     `\ص`     |    `\f`    | الصفحة التالية       |
|     000D  |     `\ج`     |    `\r`    |  إرجاع المؤشر      |
|     0020 |    `' '`     |   `' '`    |     مسافة           |