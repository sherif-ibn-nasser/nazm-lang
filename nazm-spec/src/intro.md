# بـٰسم الله الرحمن الرحيم

**نظم** هي لغة برمجة مخصصة لتسهيل تطوير الأنظمة البرمجية والتطبيقات باستخدام بنية لغوية عربية. تهدف اللغة إلى توفير بيئة برمجية مرنة تتيح للمبرمجين العرب الكتابة بلغتهم الأم، مع الحفاظ على كفاءة الأداء واستخدام مفاهيم حديثة في البرمجة.

من خلال تصميمها، تحاول نظم التغلب على التحديات اللغوية والثقافية التي تواجه البرمجيات العالمية، بتقديم تجربة برمجية محلية أكثر ملاءمة ومرونة، مع الحفاظ على التكامل مع الأنظمة والتقنيات الحالية. يوفر هذا المستند المواصفات التقنية للغة نظم، من حيث قواعدها، أنواع البيانات، الهيكلية، وآليات التحكم فيها، لتزويد المطورين بفهم شامل لطريقة عمل اللغة وكيفية استخدامها.

### عن هذا المستند

هذا المستند يمثل **المواصفات التقنية الرسمية** للغة نظم. المواصفات الفنية هي مرجع رسمي يشرح بدقة كيف تعمل اللغة من الناحية التقنية. وهي تهدف إلى تزويد المطورين والقارئين المتخصصين بالتفاصيل الضرورية لكتابة شيفرات متوافقة مع معايير اللغة، وتصميم أدوات تدعم لغة نظم.

### ما الذي تتضمنه هذه المواصفات؟

1. **قواعد اللغة**: سيتم تفصيل القواعد النحوية للغة نظم، بما في ذلك ترتيب الكلمات، أنواع البيانات، وتعريف الدوال.
2. **آليات التحكم**: تشمل العمليات المنطقية، الحلقات، الشروط، وكيفية التعامل مع الأخطاء.
3. **التكامل مع الأنظمة الأخرى**: كيفية دمج اللغة مع بيئات وأنظمة تشغيل أخرى، وكيفية التعامل مع المكتبات الخارجية.
4. **أفضل الممارسات**: نصائح وإرشادات حول كيفية كتابة شيفرة فعالة وآمنة باستخدام نظم.

### ملاحظات هامة

- **توقعات القارئ**: هذه المواصفات موجهة للمبرمجين الذين لديهم معرفة مسبقة بلغة نظم وأساسيات البرمجة بشكل عام. سيتم التركيز على التفاصيل التقنية الدقيقة دون التطرق إلى الشروحات الأساسية للمفاهيم البرمجية.
- **حالة المواصفات**: هذا المستند لا يزال قيد التطوير، ويعكس حالة اللغة كما هي في الوقت الحالي. نظرًا لأن نظم لا تزال في مراحل التطوير، فإن بعض الميزات قد تكون غير مكتملة أو قد تتغير في الإصدارات المستقبلية. لذلك، قد لا تكون هذه المواصفات نهائية وسيتم تحديثها بانتظام مع تطور اللغة.

### توصيف القواعد النحوية

يتم استخدام تدوين خاص لتوصيف قواعد لغة **نظم**، مشابه لتدوين التعبيرات العادية (Regex). هذا التدوين يساعد في تحديد تركيبة اللغة والعناصر المسموح بها عند كتابة الشيفرات. سيتم استخدام الرموز التالية لتوضيح البنية النحوية:

| الرمز                                      | الوصف                                                       | مثال                                                                                                |
| ------------------------------------------ | ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| **`'...'`** | نمط مكون من سلسلة حروف أو رموز حيث يمثل المكتوب فعلياً. |` 'دالة'`، `'؛'` |
| اسم القاعدة يبدأ بشَرطة سُفلية **`_`**     | القاعدة ناتجة من محلل الرموز (يمثل أبسط رمز ممكن)           | `_عدد_صحيح` أو الكلمات المفتاحية (`_دالة`، `_تصنيف`، إلخ)                                           |
| اسم القاعدة يبدأ بدون شَرطة سُفلية **`_`** | القاعدة ناتجة من المحلل النحوي (ناتجة من تجميع أكثر من رمز) | `جملة_احجز`                                                                                           |
| **`\|`**                                   | يعني اختيار بين بدائل                                       | `أ                                                 \| ب` يعني ظهور القاعدة أ `أو` القاعدة ب         |
| **`(...)`**                                | يستخدم لتجميع العناصر معًا لتوضيح الترتيب                   | `(أ ب)` يعني ظهور القاعدة أ `ثم` القاعدة ب                                                            |
| **`؟`**                                    | يمثل ظهور القاعدة مرة واحدة أو عدم ظهوره                    | `أ`<sup>؟</sup> يعني أن القاعدة أ قد تظهر أو لا تظهر                                                           |
| **`+`**                                    | يمثل تكرار القاعدة مرة واحدة أو أكثر                        | `أ`<sup>+</sup> يعني أن القاعدة أ يجب أن تظهر مرة واحدة على الأقل                                              |
| **`*`**                                    | يمثل تكرار القاعدة أو عدم تكرارها                           | `أ`<sup>*</sup> يعني أن القاعدة أ قد لا تظهر أو قد تظهر بتكرار. من الممكن اعتبارها مكافئة للقاعدة `(أ`<sup>؟</sup> `\| أ`<sup>+</sup>`)` |