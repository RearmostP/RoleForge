# סיכום מימוש שלב 09

מומשו מופעי Role חיים שנגישים דרך Project אחרי `load()`:

```python
project.test.hello()
project.test[0].hello()
project.test[1].hello()
assert project.test is project.test[0]
assert project.test is not project.test[1]
```

## ארכיטקטורה ובעלות

Handoff ממיר `CleanRole` ל־`RoleInput` ניטרלי ופותר את `entry.via` כמזהה אטום. חוזה Bridge כולל כעת טיפוס תוצאה משויך `Live`; Runtime מעביר את התוצאות בתוך `LoadResult<T>` לצד מטא־דאטה של גילוי והאינדקס הגלובלי. אין אובייקטי Python במודלי הקלט הניטרליים ואין ידע על Test או hello ב־Core.

ה־Python Bridge יוצר קלט Python קפוא ואז קורא לעזר הייצוג `_create_role` בתוך `roleforge/_live.py`. אם מודול היעד מגדיר מחלקת `Role` שיורשת מ־`roleforge.Role`, ה־Bridge יוצר מופע שלה עם הקלט; אחרת נוצר מופע כללי. בנאי הבסיס מספק שדות קלט לקריאה בלבד ללא העתקה ידנית. ה־receiver מאתחל מצב נוסף, והמחלקה מגדירה פעולות רגילות עם `self`. כל מסירה טוענת מקור מחדש ומייצרת מופע נפרד.

לאחר חזרה מוצלחת מה־receiver מוחזר `Py<PyAny>` בבעלות מלאה, ו־Project מחזיק את האובייקט דרך `_ProjectRoles`. המופע שומר את הקלט והמחלקה; המתודות מחזיקות את סביבת המשתנים הגלובליים שלהן. שחזור הרישום הזמני ב־`sys.modules` אינו פוגע בשימוש אחרי הטעינה, וגם מופע שנשמר לאחר שחרור Project נשאר שמיש. הבדיקות מאמתות גם איסוף של קבוצת המופעים כאשר אין עוד הפניות אליה.

## גישה וזהות

הקיבוץ משתמש בשם וב־`role_index` שכבר נקבעו ב־Core, ללא מספור מחדש. `project.test` מחזיר ישירות מופע אפס; `__getitem__` במחלקת הבסיס בוחר מאותה קבוצת מופעים. אין שכפול של מופע לצורך קיצור הגישה. שם הקובץ מזהה את הקשר Project ולא מחליף את זהות המופע.

המיפוי הדינמי הוא `name.lower()` בלבד, עבור מזהי Python תקינים שאינם מילות מפתח. שדות ומתודות Project קיימים מקבלים עדיפות. התנגשות שמות אינה נפתרת בשקט: גישה דינמית עמומה גורמת ל־AttributeError. `get_role(name, role_index=0)` מאפשר גישה בשם המקורי המדויק, גם לשמות שמורים או חריגים. שם חסר בגישה מדויקת גורם ל־KeyError; אינדקס חסר או שלילי גורם ל־IndexError; slices אינם נתמכים. `project.roles` ממשיך להחזיר tuple של RoleInfo לקריאה בלבד.

## שינויי פרוטוקול ביחס לשלב 08

- ה־receiver מקבל כעת מופע חי עם אותם שדות קלט לקריאה בלבד, ויכול להוסיף מצב פרטי למופע.
- מודול היעד יכול להצהיר על מחלקת Role אופציונלית לפי חוזה הבסיס. מחלקה לא תקינה או כשל בבניית המופע מדווחים כ־InputConversion עם הקשר המסירה.
- תוצאת Bridge הפנימית נושאת מופע בבעלות, אך ערך החזרה של `roleforge_receive` עדיין נזנח ואינו הופך לאובייקט הציבורי.
- כללי preflight, דילוג על Unknown, עצירה בכשל והיעדר rollback נשמרו. אין הפעלה אוטומטית של `start()`.

## קבצים

נוספו `roleforge/_live.py`, `anyone_py_project/test_live_roles.py` ודוח זה.

עודכנו `roleforge/python_api.rs`, `roleforge/__init__.py`, `roleforge/core/bridges/python.rs`, `roleforge/core/bridges/mod.rs`, `roleforge/core/bridges/tests.rs`, `roleforge/core/pipeline/handoff.rs`, `roleforge/core/pipeline/runtime.rs`, `roleforge/core/pipeline/runtime/tests.rs`, `roleforge/roles/Test/main.py` ו־`anyone_py_project/main.py`.

עודכנו README הראשי, כל ששת מסמכי האדם בעברית ובאנגלית, `docs/ai/CORE_IRON_RULES.md` ואינדקס הפרומפטים. הפרומפטים ההיסטוריים ודוח שלב 08 נשמרו. בתחילת העבודה כבר היו שינויים ב־Handoff, ב־Runtime, בחוזה Bridges, בבדיקות שלהם ובדוגמת Test; המימוש ממשיך אותם. תיקיית `.idea` הקיימת לא נערכה.

## בדיקות ותוצאות

נוספו 12 בדיקות Python שמכסות זהות בין קיצור הגישה ל־`[0]`, מופעים חוזרים ומשולבים, מצב נפרד, lifetime ושחרור, גישה לא תקינה, Unknown, conflict לפני import, התנגשויות שמות, receiver ישן ללא מחלקה, טעינות נפרדות, עצירה בכשל ללא rollback, מחלקה לא תקינה ודוגמת Test. בדיקת Rust נוספת מאמתת שימור תוצאות Bridge והאינדקסים דרך Runtime כשיש Unknown בין מסירות.

תוצאות:

- `cargo test`: עברו 38 בדיקות, 0 נכשלו; 0 בדיקות תיעוד.
- `cargo build`: הצליח.
- `cargo fmt --check`: עבר.
- `python -m unittest discover -s anyone_py_project -v`: עברו 27 בדיקות, 0 נכשלו, כולל בדיקות שלב 08 הקיימות.
- `git diff --check`: עבר.

לבדיקות Python הועתק DLL הבנייה המקומי אל `roleforge/_native.pyd` והוגדר PYTHONPATH לשורש הפרויקט עבור תהליכי הבדיקות. קובצי Registry הוחזרו לבתים המקוריים באמצעות cleanup/finally. נשארה אזהרת הקומפיילר הקיימת על `LoadedFile.path` שאינו נקרא בספרייה. לא בוצעו commit או push.

## החלטות שנותרו פתוחות

מדיניות שמות רחבה יותר, aliases ושמות מופעים, lifecycle של start, מסירת Rust וסביבות נוספות, רישום והתקנה ציבוריים, אריזה ניידת ו־Error/Console Managers סופיים נשארו מחוץ לשלב זה. בחירת המופע הראשון והגישה באינדקס הן כעת חוזה ממומש, ולא החלטה פתוחה.
