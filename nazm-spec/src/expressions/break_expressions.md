## تعبيرات 'قطع'

تُستخدم تعبيرات `قطع` للخروج من الحلقات التكرارية:

```nazm
دالة البداية() {
    احجز متغير م = 0؛

    طالما م < 5 {

        لو م == 3 {
            // إذا كان العدد 3، قُم بقطع التنفيذ والخروج من الحلقة التكرارية
            قطع؛
        }

        اظهر_(م)؛
        م += 1؛
    }

}
```

نوع تعبيرات `قطع` هو النوع [`!!`]().

### النحو

> `تعبير_قطع` :
>
> &emsp; '**`قطع`**'