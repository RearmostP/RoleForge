# היכרות עם RoleForge

[English](../en/README.md) · [עמוד הפרויקט](../../../README.md) · [יצירת Roles](CREATING_ROLES.md) · [כללי הברזל](IRON_RULES.md)

RoleForge מאפשר לשלב כמה Roles עצמאיים בפורמט מקור משותף. במקום שפת DSL אחת גדולה, כל Role יכול להגדיר את התחביר, ה־tokenizer, ה־parser, הבדיקות, ההתנהגות וה־API שהוא צריך. אין חובה לממש את כולם: גם Role שרק מדפיס את הקלט שלו הוא Role תקין.

ה־Core, שכתוב ב־Rust, מבין את גבולות ה־Roles ואת כללי הניתוב. כל Role אחראי למשמעות של השפה שלו. למשל, Core יכול לזהות בלוק של `Test` בלי לדעת מה המשמעות של `hello = first` בתוכו. הפרויקט עדיין בפיתוח ואינו מוצג כמוכן לשימוש בייצור.

## איך המידע מגיע ל־Role

```text
.rfg source
    → Core: Loader + Main Tokenizer
    → Registry + Dispatcher
    → Handoff preflight
    → RoleInput + resolution of entry.via
    → Bridge → entry.target
    → roleforge_receive(role)
```

ה־Loader קורא את הקובץ, וה־Main Tokenizer מגלה את המופעים. ה־Registry וה־Dispatcher קובעים לאן לנתב אותם. לפני מסירה מתבצעת בדיקת קונפליקטים על התוצאה המלאה. אחריה Handoff יוצר `RoleInput`, מאתר את ה־Bridge שנבחר ומוסר לו את הנתונים ואת היעד.

Runtime מתאם את הפעולות; רכיב מרכזי אינו מפעיל בעצמו את הרכיב המרכזי הבא. אין Main Parser: אם צריך לנתח את השפה של Role, זה תפקידו של ה־Role.

ה־Python Bridge כתוב ב־Rust ומשתמש ב־PyO3, כמו ממשק ה־Python הציבורי. מודלי Core נשארים נתוני Rust רגילים; ה־Bridge מתאים אותם לאובייקט Python לקריאה בלבד. אחריות המסירה של Core מסתיימת כשה־receiver חוזר בהצלחה. לערך החזרה שלו אין משמעות פרוטוקולית.

## הרצת הדוגמה הקיימת

נדרשים עותק מקור של הפרויקט, Python בגרסה 3.10 ומעלה, Rust/Cargo שתומכים ב־edition 2024, וכלי הקומפילציה והקישור הנדרשים בסביבת Rust שלכם. הבנייה משתמשת ב־Maturin דרך `pyproject.toml`; ייתכן ש־pip יוריד תלויות בנייה.

משורש הפרויקט יוצרים סביבה וירטואלית:

```sh
python -m venv .venv
```

מפעילים אותה ב־PowerShell:

```powershell
.\.venv\Scripts\Activate.ps1
```

או ב־shell תואם POSIX:

```sh
. .venv/bin/activate
```

לאחר מכן בונים ומתקינים את עותק המקור ומריצים את הדוגמה:

```sh
python -m pip install -e .
python -u anyone_py_project/main.py
```

[הדוגמה](../../../anyone_py_project/main.py) מוצאת את קובץ ה־`.rfg` שלצדה ואינה מייבאת את ה־Role ידנית. [ה־Registry הדינמי](../../../roleforge/core/storage/dynamic_roles.json) כבר כולל את `Test`, עם `via: "python"` ו־`target: "Test/main.py"`.

ה־receiver מדפיס שני מופעים: אינדקסים `0/0` ו־`1/1`, שורות הצהרה `1` ו־`5`, וגופים שמכילים `hello = first` ו־`hello = second`. מופיע גם פלט דיבוג זמני של Core. הדוגמה מוכיחה קבלה של מידע; היא אינה מדגימה parser או הפעלת `start()`.

אם `import roleforge` נכשל, ודאו שאתם מריצים את אותו Python שבו התקנתם את הפרויקט. עותק מקור שלא נבנה אינו מספק את ההרחבה המקומית. לאחר שינוי Rust יש לבנות מחדש. גם לאחר העברת עותק המקור למיקום אחר צריך לבנות מחדש, כי מיקום ה־Registry נקבע כיום בזמן הבנייה.

## מה load() מחזיר

```python
from roleforge import load

project = load("anyone_py_project/test.rfg")  # From the repository root.
for info in project.roles:
    print(info.name, info.index, info.role_index, info.status)
```

ל־`Project` יש שני שדות לקריאה בלבד: `path` ו־`roles`. השדה `roles` הוא tuple של רשומות `RoleInfo`, גם הן לקריאה בלבד, לפי סדר המקור. בכל רשומה קיימים `name`, `index`, `role_index`, `body`, `declaration_line`, `status`, `entry`, `builtin_entry` ו־`dynamic_entry`.

אלו נתוני גילוי וניתוב, ולא מופעי Role שאפשר להפעיל. הם גם אינם אובייקט ה־`RoleInput` שמתקבל ב־receiver: שם מידע המקור נמצא תחת `role.source`. ברשומה שנפתרה, `entry` הוא נתיב היעד שנפתר; ברשומה עם קונפליקט זמינים שני הנתיבים המתנגשים. שדות נתיב שאינם רלוונטיים מכילים `None`.

## ניתוב ושגיאות במימוש הנוכחי

| מצב | ההתנהגות של load כיום |
| --- | --- |
| השם רשום בדיוק ב־Registry אחד | המופע מועמד למסירה לפי סדר המקור. |
| השם אינו רשום באף Registry | מדווח Unknown והמופע מדולג; מופעים אחרים שנפתרו יכולים להמשיך. |
| השם רשום בשני ה־Registries | כל הקונפליקטים מדווחים, לא נמסר שום מופע בטעינה הזו, ומוחזר Project עם המטא־דאטה. אין עדיפות לאחד הרישומים. |
| Bridge לא מוכר, כשל בטעינת היעד, receiver חסר או לא callable, או חריגה ב־receiver | נזרקת `RuntimeError` עם הקשר והמסירות הבאות נעצרות. מסירות שכבר הצליחו אינן מבוטלות. |
| תחביר חיצוני לא תקין | נזרקת `ValueError` עם שורת המקור. |
| כשל קריאה או פענוח של המקור או ה־Registry | נזרקת שגיאת קלט/פלט מתאימה ב־Python. |

`status == "resolved"` מציין שנמצא ניתוב, ולא מאשר מסירה: קונפליקט במקום אחר יכול למנוע את כל המסירות. הדיווח על Unknown ו־Conflict ופלט הדיבוג הם מדיניות זמנית, לא ה־Error Manager או ה־Console Manager הסופיים.

## מה קיים ומה עדיין פתוח

ממומשים: Loader, Main Tokenizer, Registry, Dispatcher, Runtime, preflight, Handoff, פתרון Bridges, מודל RoleInput ניטרלי, Python Bridge, ממשק `load()`, מטא־דאטה של מקור ואינדקסים גלובליים ולפי שם. ה־Role הדינמי `Test` מפעיל מסירה אמיתית ל־Python.

טרם מומשו או הוגדרו סופית: מסירה ל־Rust, Bridges נוספים, מחזור חיים של `start()` בניהול Core, גישה דינמית כמו `project.directory` או `project.test[1]`, aliases ושמות מופעים, API להתקנה ולהסרה של Roles, ו־Error/Console Managers סופיים. הארכיטקטורה מאפשרת הוספת Bridges בעתיד, אבל אינה מבטיחה מנגנון מסוים עבורם.

ה־Registry של ה־Roles המובנים ריק כרגע. `roleforge/builtin_roles/` משמש בסיס ליעדים מובנים; `Test` הוא Role דינמי לצורכי פיתוח. Bridge מובנה ו־Role מובנה הם שני מושגים שונים.

אחסון ה־Registry ובסיסי הנתיבים קשורים כרגע לעץ המקור דרך `CARGO_MANIFEST_DIR` של Rust בזמן הבנייה. כללי הפצה ונתיבי runtime ניידים עדיין פתוחים; ההוראות כאן מיועדות לפיתוח מתוך עותק מקור.

## בדיקות ומסמכים לתורמים

לאחר התקנת עותק המקור, מריצים משורש הפרויקט, לפי הסדר:

```sh
cargo test
python -m unittest discover -s anyone_py_project -v
```

בדיקות המסירה ב־Python מחליפות זמנית את קובצי ה־Registry ומחזירות את הבתים המקוריים בסיום. אין להריץ אותן במקביל לטעינות או לבדיקות אחרות שמשתמשות באותם קבצים.

לפני שינוי ארכיטקטורה כדאי לקרוא את [כללי הברזל למפתחים](IRON_RULES.md). [כללי הברזל ל־AI](../../ai/CORE_IRON_RULES.md) מספקים פירוט נוסף. [דוח שלב 08](../../ai/STAGE_08_IMPLEMENTATION.md) מתעד את אותה נקודת פיתוח, ו[אינדקס הפרומפטים](../../../roleforge/core/pipeline/prompts/README.md) מסביר את ההיסטוריה. פרומפט היסטורי אינו גובר על התיעוד העדכני או על כללי הארכיטקטורה המאושרים.
