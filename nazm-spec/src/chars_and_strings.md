## القيم النصية

### الحروف

يتم تعريف الحرف بين علامتي تنصيص مُفرَدتين (`'`):

```nazm
دالة البداية() {
   اظهر_('أ')؛
   اظهر_('ب')؛
   اظهر_('ج')؛
   اظهر_('د')؛
}
```

**أحرف التحكم**

أحرف التحكم هي رموز خاصة (تبدأ بالشَرطة المائلة العكسية **\\**) تُستخدم للتحكم في طريقة عرض البيانات أو لتنفيذ عمليات معينة دون ظهورها كجزء من النص، وهى:

| حرف التحكم | في لغة C | الوظيفة                                                           |
| :--------: | :------: | ----------------------------------------------------------------- |
|    \\"     |   \\"    | استخدام علامة التنصيص المزدوجة كحرف أصلي                          |
|    \\'     |   \\'    | استخدام علامة التنصيص المُفردة كحرف أصلي                          |
|    \\\\    |   \\\\   | استخدام علامة الشرطة المائلة العكسية كحرف أصلي                    |
|    \\س     |   \\n    | الانتقال إلى سطر جديد                                             |
|    \\ف     |   \\t    | التحرك مسافة أفقية                                                |
|    \\ر     |   \\v    | التحرك مسافة رأسية                                                |
|    \\خ     |   \\b    | التحرك مسافة للخلف (يقوم بمسح آخر حرف مثل زر الbackspace)         |
|    \\ص     |   \\f    | الانتقال إلى الصفحة التالية أثناء الكتابة (يظهر أثره في الطابعات) |
|    \\ج     |   \\r    | إرجاع المؤشر إلى بداية السطر، وبدء الكتابة منه                    |
|    \\0     |   \\0    | إنهاء النص                                                        |

**ترميز يونيكود**

في `نظم`، يمكن استخدام ترميز يونيكود لتمثيل مجموعة واسعة من الأحرف من لغات مختلفة ورموز متنوعة. يُعتبر ترميز يونيكود طريقة موحدة لتمثيل النصوص بطريقة تضمن ظهور الأحرف بشكل صحيح عبر مختلف الأنظمة والأجهزة.

فقط قُم بكتابة علامة الشرطة المائلة الخلفية (`\\`) ثم حرف الياء (`ي`)، ثم ضع كود رمز اليونيكود المكون من أربع أرقام بالنظام السُداسي عشر:

```nazm
دالة البداية() {
   اظهر("هذا حرف الألف باليونيكود: ")؛
   اظهر_('\ي0627')؛

   اظهر("وهذا حرف التاء باليونيكود: ")؛
   اظهر_('\ي062A')؛

   اظهر("وهذا رمز باي باليونيكود: ")؛
   اظهر_('\ي03C0')؛
}
```

عند التنفيذ:

```shell, rtl
هذا حرف الألف باليونيكود: أ
وهذا حرف التاء باليونيكود: ت
وهذا رمز باي باليونيكود: π
```

> [!ملحوظة]
> بعض الرموز في اليونيكود التي تمثل أشياء تخالف الإسلام (صُلبان، موسيقى، شذوذ، إلخ) أو بعض الرموز الغير مدعومة، سيقوم المترجم بإظهار خطأ إذا تم اكتشاف أحد تلك الرموز في الكود (حتى لو كان مكتوباً بترميز اليونيكود).

**حجم الحرف**

الحروف في `نظم` تشغل حجم 4 بايت في الذاكرة، وتُمثِّل ما يُعرف **بقيمة يونيكود القياسية** للحرف.
هذا التصميم يسمح بتمثيل جميع الأحرف بما في ذلك تلك التي تحتاج إلى أكثر من 1 أو 2 بايت كما هو الحال في نظام UTF-8.
على الرغم من أن بعض الأحرف في UTF-8 قد تشغل أحجامًا مختلفة (مثل 1 بايت للأحرف الإنجليزية أو 2 بايت للأحرف العربية)،
فإن تخصيص 4 بايت لكل حرف في نظم يضمن التوافق الكامل مع جميع رموز يونيكود.

### المتون

في لغة `نظم`، المتون هي السلاسل النصية التي تُكتب بين علامتي تنصيص مزدوجتين (`"`). المتون تُستخدم لتمثيل النصوص المكونة من سلسلة من الأحرف، سواء كانت أحرفًا أبجدية، أرقامًا، رموزًا، أو حتى مسافات وأحرف تحكم.

```nazm
دالة البداية() {
    احجز كلام: #متن = "هذا متن"؛
}
```

### النحو

> \_حرف :
>
> &emsp; '**`'`**' &nbsp; **\_حرف_أساسي** &nbsp; '**`'`**'
>
> \_متن :
>
> &emsp; '**`"`**' &nbsp; **\_حرف_أساسي**<sup>\*</sup> &nbsp; '**`"`**'
>
> \_حرف_أساسي :
>
> &emsp; **\_حرف_تحكم** &nbsp; \| &nbsp; **.**
>
> \_حرف_تحكم :
>
> &emsp; '**`\`**' &ensp; (
> &emsp; &ensp; '**`\`**'
>
> &emsp; &emsp; &emsp; &emsp; \| &nbsp; '**`"`**'
>
> &emsp; &emsp; &emsp; &emsp; \| &nbsp; '**`'`**'
>
> &emsp; &emsp; &emsp; &emsp; \| &nbsp; '**`0`**'
>
> &emsp; &emsp; &emsp; &emsp; \| &nbsp; '**`س`**'
>
> &emsp; &emsp; &emsp; &emsp; \| &nbsp; '**`ف`**'
>
> &emsp; &emsp; &emsp; &emsp; \| &nbsp; '**`ر`**'
>
> &emsp; &emsp; &emsp; &emsp; \| &nbsp; '**`خ`**'
>
> &emsp; &emsp; &emsp; &emsp; \| &nbsp; '**`ص`**'
>
> &emsp; &emsp; &emsp; &emsp; \| &nbsp; '**`ج`**'
>
> &emsp; &emsp; &emsp; &emsp; \| &nbsp; '**`ي`**' [\_رقم_16](numbers.md#النحو) <sup>4</sup>
>
> &emsp; &emsp; &emsp; )