# סיכום ניקוי שלב 10

הופרדו בדיקות Rust מקובצי המימוש והועבר פרויקט הצרכן המדומה אל `external_test_project/`. לא שונתה במכוון התנהגות RoleForge, לא הורחבה חשיפת API ולא בוצעו שינויי אריזה או פרסום.

## העברת בדיקות Rust

| קובץ שהכיל בדיקות inline | קובץ הבדיקות החדש |
| --- | --- |
| `roleforge/core/registry.rs` | `roleforge/core/registry/tests.rs` |
| `roleforge/core/pipeline/loader.rs` | `roleforge/core/pipeline/loader/tests.rs` |
| `roleforge/core/pipeline/dispatcher.rs` | `roleforge/core/pipeline/dispatcher/tests.rs` |
| `roleforge/core/pipeline/tokenizer/tokenizer.rs` | `roleforge/core/pipeline/tokenizer/tokenizer/tests.rs` |

בקובצי המימוש נשארה הצהרת `#[cfg(test)] mod tests;`. עזרי הבדיקות הועברו עם המודולים, כולל import של PathBuf שהיה בקובץ Dispatcher. בדיקות Runtime ו־Bridges כבר היו בקבצים נפרדים. לא הוסרו בדיקות. השוואה למקור לאחר rustfmt אישרה שגופי הבדיקות וקוד הריצה נשמרו.

## פרויקט הצרכן והפניות

כל קובצי פרויקט הצרכן הועברו אל `external_test_project/`, ונוסף בו README קצר שמסביר את תפקידו ואת פקודות ההרצה. בדיקות Rust פנימיות נשארו בספרייה תחת cfg(test); הפרויקט החיצוני מכיל בדיקות Python ציבוריות, דוגמאות וקובצי מקור.

כל ההפניות הפעילות לשם הקודם עודכנו בקבצים הבאים:

- `README.md`: מפת המאגר וקישורי הדוגמאות.
- `docs/human/en/README.md` ו־`docs/human/he/README.md`: פקודות הרצה, בדיקות, קישורים ודוגמת load.
- `docs/human/en/CREATING_ROLES.md` ו־`docs/human/he/CREATING_ROLES.md`: קישורי main.py ו־test.rfg.
- `external_test_project/test_python_api.py`: פקודת הבדיקה בתיאור המודול.
- `roleforge/core/pipeline/runtime/tests.rs`: נתיב קובץ הבדיקה.

נוספו הסברי גבולות הבדיקות בתיעוד הראשי ובכללי הברזל ל־AI, ושלב 10 נוסף לאינדקס הפרומפטים. האזכורים בדוחות שלבים 08 ו־09 ובפרומפטים ההיסטוריים נשמרו במכוון כתיעוד של זמנם. פרומפט שלב 10 שכבר היה במצב staged/modified לפני העבודה לא נערך.

## אימות

- `cargo test`: עברו 38 בדיקות; 0 נכשלו, 0 בדיקות תיעוד.
- `cargo build`: הצליח.
- `cargo fmt --check`: עבר.
- `python -m unittest discover -s external_test_project -v`: עברו 27 בדיקות; 0 נכשלו.
- `python -u external_test_project/main.py`: הצליח; הודפסו קריאות hello למופע 0 פעמיים ולמופע 1 פעם אחת.
- `git diff --check`: עבר.
- השוואת קובצי הצרכן למקור אישרה שימור בדיקות ונתוני fixtures, פרט לעדכון שם התיקייה בפקודת הבדיקה.

בדיקות Python השתמשו בהרחבה המקומית שנבנתה מחדש וב־PYTHONPATH של שורש המאגר. בדיקות שמשנות Registry החזירו את תוכנו המקורי. לא שונו Cargo.toml, pyproject.toml או חוזי זמן הריצה. לא בוצעו commit או push.

## בעיות שנותרו

Windows מחזיק נעילה של תהליך אחר על התיקייה הישנה והריקה `anyone_py_project/`. כל תוכנה כבר הועבר ואין בה עוד פרויקט פעיל, אך מחיקת התיקייה עצמה עדיין נכשלת. יש לשחרר את התהליך שמחזיק בה ואז להסיר את התיקייה הריקה. נשארה גם אזהרת הבנייה הקיימת על `LoadedFile.path` שאינו נקרא בקוד הספרייה.
