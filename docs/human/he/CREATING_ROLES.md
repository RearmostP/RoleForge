# יצירת Role דינמי ב־Python

[לתיעוד הראשי](README.md) · [English](../en/CREATING_ROLES.md) · [כללי הברזל](IRON_RULES.md)

Python הוא כרגע סביבת המימוש היחידה שיש עבורה מסירת Role עובדת. התחילו עם הוראות ההתקנה ב[מבוא](README.md). אין כרגע CLI להתקנת Roles או API לרישום: הרישום מתבצע בעריכה ידנית של JSON.

עותק המקור כבר כולל את `Test`. הדוגמה הבאה מציגה גרסה מינימלית מלאה של אותו מבנה. ל־Role חדש בחרו שם חדש ושמרו את יתר הרשומות ב־Registry.

## 1. כותבים receiver

האזור הנוכחי ל־Roles דינמיים הוא `roleforge/python/roleforge/roles/`:

```text
roleforge/python/roleforge/
└── roles/
    └── Test/
        └── main.py
```

בקובץ `roleforge/python/roleforge/roles/Test/main.py`:

```python
from roleforge import Role as BaseRole

class Role(BaseRole):
    def hello(self):
        self.calls += 1
        return self.body

def roleforge_receive(role):
    role.calls = 0
    print(role.name)
    print(role.index)
    print(role.role_index)
    print(repr(role.body))
    print(role.source.declaration_line)
```

`roleforge_receive` היא נקודת הקבלה המחייבת. Core אינו מחפש במקומה `main`, `run`, `receive` או `start`. משתמשים בפונקציה סינכרונית רגילה שמקבלת אובייקט אחד. ביצוע אסינכרוני של Roles אינו ממומש.

קבלה פירושה ש־RoleForge מסר מופע שגילה במקור. היא אינה הוראה ל־Core להפעיל את הפעולות שה־Role מציע למשתמש. `start()` הוא פעולה נפרדת ואינו נקרא אוטומטית. חזרה רגילה מה־receiver נחשבת לקבלה מוצלחת; ערך החזרה אינו משמש את הפרוטוקול. חריגה מכשילה את המסירה.

[המימוש הקיים של Test](../../../roleforge/python/roleforge/roles/Test/main.py) מוסיף כותרות להדפסות וחושף `hello()`. מחלקת `Role` האופציונלית במודול היעד יורשת מ־`roleforge.Role`. הבנאי המורש מתאים את RoleInput בלי העתקת שדות ידנית; מאתחלים מצב ב־`roleforge_receive`. יעד ללא מחלקת `Role` מקבל מופע חי כללי. אם מוגדרת מחלקה כזו, היא חייבת לרשת ממחלקת הבסיס ולקבל את חוזה הבנייה עם קלט אחד. ה־Bridge אינו מסיק API מפונקציות אחרות במודול ואינו מפרש שמות פעולות.

## 2. רושמים את היעד

עורכים את `roleforge/python/roleforge/core/storage/dynamic_roles.json`:

```json
{
  "Test": {
    "entry": {
      "via": "python",
      "target": "Test/main.py"
    }
  }
}
```

| שדה | משמעות |
| --- | --- |
| `Test` | השם שצריך להתאים להצהרת `@role Test` שהתגלתה במקור. |
| `via` | מזהה מדויק של Bridge רשום. כרגע `python` הוא הרישום המובנה היחיד. זה אינו שדה כללי שמצהיר על שפת תכנות. |
| `target` | יעד שה־Bridge הנבחר מפרש; במסירת Python מדובר כרגע בקובץ מקור של Python. |

הבחירה ב־Bridge אינה נגזרת מהסיומת `.py`. מזהה שלא נרשם יגרום לכשל גם אם הקובץ מכיל Python תקין. רישום ה־Bridges עצמם נעשה פנימית ב־Rust, ולא דרך API ציבורי של Python.

כללי פתרון היעד כיום:

- יעד דינמי יחסי מחושב ביחס ל־`roleforge/python/roleforge/roles/`.
- יעד מובנה יחסי מחושב ביחס ל־`roleforge/python/roleforge/builtin_roles/`.
- יעד מוחלט נשאר מוחלט.

בעותק המקור `Test/main.py` מצביע על `roleforge/python/roleforge/roles/Test/main.py`. בחבילה מותקנת בסיסי היעדים ואחסון ה־Registry נמצאים תחת `Path(roleforge.__file__).resolve().parent`, ללא תלות בעותק המקור או בתיקיית העבודה. לניהול ידני עורכים את קובצי ה־Registry ואת תיקיות ה־Roles בחבילה המותקנת. התקנת Wheel מחדש עלולה לדרוס שינויים ידניים.

שם צריך להיות רשום רק באחד הקבצים `dynamic_roles.json` ו־`builtin_roles.json`. רישום בשניהם יוצר Conflict ומונע את כל המסירות באותה טעינה. לעומת זאת, כמה הצהרות של אותו Role במקור הן מצב תקין ואינן קונפליקט רישום.

## 3. יוצרים מקור וטוענים אותו

שמרו את התוכן הבא כ־`test.rfg` בקידוד UTF-8 בתיקייה שממנה תריצו את קוד ה־Python הבא:

```text
@role Test

hello = world
number = 123
```

```python
from roleforge import load

project = load("test.rfg")
print(project.test.hello())
```

אין צורך לייבא את ה־Role ידנית. `load()` קורא את הקובץ, מגלה את `Test`, פותר את הרשומה שלו, בודק קונפליקטים, יוצר RoleInput, בוחר את ה־Python Bridge, טוען את היעד וקורא ל־`roleforge_receive(role)`.

בדוגמה הזו `name` הוא `Test`, שני האינדקסים הם `0`, ו־`source.declaration_line` הוא `1`. עם סיומות שורה LF ושורה חדשה בסוף, הגוף הוא `'\nhello = world\nnumber = 123\n'`. מקור עם CRLF שומר על CRLF. השורה הריקה בתחילת הגוף היא חלק מהקלט, ו־Core אינו מפרש את ההשמות שבתוכו.

נתיב יחסי שמועבר ל־`load()` מחושב ביחס לתיקיית העבודה של הקורא. זה כלל נפרד מפתרון יעדי Registry. [main.py שבפרויקט](../../../external_test_project/main.py) משתמש ב־`Path(__file__).with_name("test.rfg")` כדי למצוא את המקור שלצדו גם בהרצה מתיקייה אחרת.

## 4. מכירים את RoleInput

Python מקבל מופע חי ששדות הקלט שלו ומטא־דאטה המקור המקונן נשארים לקריאה בלבד. שדות נוספים יכולים להחזיק מצב נפרד לכל מופע:

| שדה | משמעות |
| --- | --- |
| `role.name` | השם שהתגלָה בהצהרת `@role Name`. |
| `role.index` | המיקום בין כל ה־Roles בקובץ, החל מ־0. |
| `role.role_index` | מספר ההופעה בין Roles עם אותו שם בדיוק, החל מ־0. |
| `role.body` | הגוף לאחר עיבוד ההערות הקיים ברמת Core; התחביר של ה־Role נשאר ללא פירוש. |
| `role.source.declaration_line` | שורת ההצהרה בקובץ המקור, החל מ־1. |

Core קובע את האינדקסים בזמן הגילוי, גם ל־Roles לא מוכרים או עם קונפליקט. Bridges אינם מחשבים אותם מחדש. בגבול Handoff מתבצעת המרה מפורשת מנתוני הגילוי `CleanRole` לחוזה RoleInput הניטרלי. ה־Python Bridge מתאים את הייצוג ל־Python.

למשל, קטע המקור הבא מדגים גילוי בלבד; הוא אינו רושם את השמות ב־Registry:

```text
@role Directory
@role Config
@role Directory
```

| שם | index | role_index |
| --- | --- | --- |
| Directory | 0 | 0 |
| Config | 1 | 0 |
| Directory | 2 | 1 |

## 5. מוסרים כמה מופעים

[test.rfg שבפרויקט](../../../external_test_project/test.rfg) מכיל:

```text
@role Test

hello = first

@role Test

hello = second
```

רישום אחד מספיק כדי למסור את שני המופעים בנפרד לאותו receiver, לפי סדר המקור. ערכי `(index, role_index)` הם `(0, 0)` ו־`(1, 1)`, ושורות ההצהרה הן `1` ו־`5`. עם סיומות LF הגופים הם `'\nhello = first\n\n'` ו־`'\nhello = second\n'`.

הביטוי `project.test is project.test[0]` מחזיר אמת. `project.test[1]` הוא אובייקט חי נפרד, ו־`hello()` משתמש בגוף ובמצב שלו. `project.roles` נשאר מטא־דאטה של גילוי וניתוב מסוג `RoleInfo`, בנפרד מ־RoleInput הניטרלי ומהמופעים החיים. שם קובץ המקור מזהה את הקשר ה־Project; הזוג `(name, role_index)` מזהה מופע בתוכו, ו־`index` שומר על הסדר הגלובלי.

הגישה הדינמית משתמשת רק ב־`name.lower()`, ללא המרה ל־snake_case. נחשפים כך רק מזהי Python תקינים שאינם מילות מפתח. לשדות ולמתודות הקיימים ב־Project יש עדיפות. התנגשות בין שמות לאחר ההמרה גורמת ל־`AttributeError`; לגישה בשם המקורי המדויק משתמשים ב־`project.get_role("ExactName", role_index)`, גם עבור שמות שמורים או חריגים. שדה דינמי חסר גורם ל־`AttributeError`, שם מדויק חסר ל־`KeyError`, ואינדקס שאינו קיים, כולל שלילי, ל־`IndexError`. האינדקס הוא הזהות המקומית שקבע Core ולא מיקום ברצף Python; אין תמיכה ב־slices. מדיניות שמות רחבה ו־aliases נשארות פתוחות.

למימוש מתקדם שמנהל ייצוג משלו, `role.role_input` חושף את הקלט הקפוא; אפשר לשמור אובייקטים מותאמים במצב של ה־Role בלי לשנות את ההתעלמות מערך החזרה של ה־receiver.

## כללי התחביר החיצוני

- הצהרת `@role Name` מתחילה בעמודה 0. בתוך גוף, `@role` מוזח הוא טקסט רגיל.
- בלוק מסתיים בהצהרה התקינה הבאה או בסוף הקובץ. אין הוראת `@end`.
- לפני ה־Role הראשון מותרות שורות ריקות והערות של שורה שלמה; תוכן אחר שם הוא שגיאה. גם הצהרה ללא שם היא שגיאה.
- הערות `#` של שורה שלמה, כולל מוזחות, מוסרות תוך שימור מבנה ירידות השורה. `#` באמצע שורה בגוף נשאר ללא שינוי. בשורת הצהרה מותרת הערה, למשל `@role Test # comment`.
- גופים ריקים ושמות חוזרים מותרים. Core אינו מנתח מחרוזות בתוך גוף ה־Role; הצהרה בעמודה 0 עדיין יוצרת גבול גם שם.

## מגבלות טעינה ואבחון תקלות

ה־Bridge טוען קובץ מפורש בתוך תהליך ה־Python הנוכחי. בכל מסירה הוא טוען את המקור מחדש, משתמש בנתיב הקנוני המלא כדי למנוע התנגשות בין שמות קבצים זהים, ומשחזר בסיום את הרישום הזמני ב־`sys.modules`. אין להסתמך על שימור משתנים גלובליים בין מסירות. המופע החי מחזיק את הקלט, המחלקה וסביבת הפונקציות שלו אחרי חזרת `load()`, גם אם שומרים אותו ומשחררים את Project. אין גילוי חבילות, חיפוש Roles או הוספת תיקיית היעד ל־`sys.path`.

Bridge לא מוכר, יעד שלא ניתן לטעון, receiver חסר, receiver שאינו callable וחריגה ב־receiver נשמרים כקטגוריות כשל נפרדות ומועברים החוצה כ־`RuntimeError` עם הקשר. מתקנים את הרישום, הקובץ או ה־receiver שעליהם השגיאה מצביעה; אין ניסיון ב־receiver או ב־Bridge חלופי. התנהגות Unknown ו־Conflict מתוארת ב[מבוא](README.md).

מסירת Roles ל־Rust אינה ממומשת בכוונה; פרוטוקול הטעינה והקבלה שלה לא נבחר. גם aliases, API להתקנה, מחזור חיים סופי ותשתית שגיאות וקונסולה סופית הם נושאים פתוחים, ולא יכולות שהמדריך מסתמך עליהן.
