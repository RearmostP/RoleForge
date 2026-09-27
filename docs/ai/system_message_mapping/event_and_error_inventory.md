# מיפוי האירועים והשגיאות — RoleForge

תאריך: 22 בספטמבר 2026. מיפוי לפי פרומפט 12, על בסיס הקוד המקומי הפעיל.

## 1. היקף, שיטה ותוצאה

נמצאו **54 נקודות משמעותיות במלאי**, המסומנות D001–D054. הספירה כוללת זיהוי תנאים, גבולות המרה והפצה, החלטות המשך, פלט, fallback והשמטת מידע. אין מדובר ב־54 סוגי שגיאה עצמאיים: אותו תנאי יכול להיות מזוהה במקום אחד ומוצג במקום אחר, ואלה מתועדים בנפרד. פעולות סמוכות בעלות אותו ייצוג ואותה השפעה מאוגדות בשורה אחת, עם פירוט מיקומן. המזהים זמניים לצורכי הדיון בלבד, ואינם הצעה לקודי שגיאה.

נסקרו כל קובצי Rust ו־Python הפעילים תחת `roleforge/src/`, `roleforge/python/roleforge/` ו־`external_test_project/`, לרבות בדיקות קיימות, קובצי הרישום ודוגמאות המקור. הנתיבים הללו הם המיקום הנוכחי של `src/` ושל `python/roleforge/` המוזכרים בפרומפט. נבדקו גם הגדרות האריזה ו־[כללי הארכיטקטורה](../CORE_IRON_RULES.md), בעיקר סעיפים 6, 15–21 ו־23–24. דוחות היסטוריים אינם בסיס לקביעת התנהגות.

זהו **ניתוח סטטי של הקוד ושל הבדיקות הקיימות**; לא הורצו בדיקות בזמן הכנת הדוח. לפיכך אזכור בדיקה בהמשך מציין ראיה כתובה להתנהגות המצופה, ולא תוצאת הרצה חדשה. לא שונו קוד, רישומים או התנהגות. הדוח אינו קובע חומרה עתידית ואינו מתכנן מערכת Error/Event חדשה.

נקודת הייחוס של Git בזמן הסקירה: `92f397b05829fd0189145a6700d3bf5776bc5be4`. היו שינויים מקומיים קיימים, בעיקר בתיעוד; המיפוי מתייחס לתוכן המקומי שנקרא, ולא רק ל־commit זה. מספרי השורות להלן נכונים למועד הסקירה.

הממצאים המרכזיים:

- Unknown מדולג ומאפשר מסירות אחרות, אם אין Conflict ואם הפלט מצליח.
- Conflict מפסיק את כל שלב המסירה לפני טעינת יעד כלשהו, אך בדרך הרגילה `load()` מחזיר `Project` עם מטא־נתונים וללא מופעים חיים.
- כשל Bridge או כשל כתיבת פלט מפסיקים את `load()`; השפעות של מסירות קודמות אינן מתבטלות.
- חריגות Python בתוך ה־Bridge מומרות בדרך כלל למחרוזות, ובהמשך ל־`RuntimeError` חדש.
- יש פלט Core זמני ישיר, פלט של Test Role, ופלט נפרד בדוגמת הצרכן.

## 2. מפתח למיקומי הקוד

הקישורים יחסיים למיקום דוח זה. בטבלאות, `API:91` פירושו שורה 91 בקובץ API.

| קיצור | קובץ |
|---|---|
| API | [roleforge/src/python_api.rs](../../../roleforge/src/python_api.rs) |
| Registry | [roleforge/src/core/registry.rs](../../../roleforge/src/core/registry.rs) |
| Loader | [roleforge/src/core/pipeline/loader.rs](../../../roleforge/src/core/pipeline/loader.rs) |
| Scanner | [roleforge/src/core/pipeline/tokenizer/role_scanner.rs](../../../roleforge/src/core/pipeline/tokenizer/role_scanner.rs) |
| Tokenizer | [roleforge/src/core/pipeline/tokenizer/tokenizer.rs](../../../roleforge/src/core/pipeline/tokenizer/tokenizer.rs) |
| Dispatcher | [roleforge/src/core/pipeline/dispatcher.rs](../../../roleforge/src/core/pipeline/dispatcher.rs) |
| Runtime | [roleforge/src/core/pipeline/runtime.rs](../../../roleforge/src/core/pipeline/runtime.rs) |
| Handoff | [roleforge/src/core/pipeline/handoff.rs](../../../roleforge/src/core/pipeline/handoff.rs) |
| Bridges | [roleforge/src/core/bridges/mod.rs](../../../roleforge/src/core/bridges/mod.rs) |
| PythonBridge | [roleforge/src/core/bridges/python.rs](../../../roleforge/src/core/bridges/python.rs) |
| Debug | [roleforge/src/core/pipeline/final_core_debug.rs](../../../roleforge/src/core/pipeline/final_core_debug.rs) |
| Live | [roleforge/python/roleforge/_live.py](../../../roleforge/python/roleforge/_live.py) |
| Init | [roleforge/python/roleforge/__init__.py](../../../roleforge/python/roleforge/__init__.py) |
| TestRole | [roleforge/python/roleforge/roles/Test/main.py](../../../roleforge/python/roleforge/roles/Test/main.py) |
| Example | [external_test_project/main.py](../../../external_test_project/main.py) |

## 3. המלאי המלא

בעמודת הפלט, „אין” פירושו שאין הדפסה ייעודית של התנאי מצד RoleForge בנקודה זו. חריגה שאינה נתפסת עשויה להיות מוצגת ב־traceback על ידי Python; זו אינה הדפסה ישירה של הרכיב. קוד יעד יכול להדפיס בעצמו לפני שהוא נכשל. „כשל טעינה” פירושו שאין `Project` מוחזר מהקריאה.

### 3.1 אתחול, קבצים ותחביר

| ID | רכיב ומיקום | תנאי | ייצוג נוכחי | השפעה נוכחית | פלט נוכחי | מגיע למשתמש? |
|---|---|---|---|---|---|---|
| D001 | API:93–100 | כשל בייבוא החבילה או pathlib, קריאת `__file__`, בניית Path, resolve, parent או חילוץ PathBuf | `PyErr` דרך `?` | כשל טעינה לפני קריאת Registry | אין | כן, החריגה מגבול Python |
| D002 | Registry:44; API:101 | קובץ `builtin_roles.json` חסר, לא קריא או אינו UTF-8 תקין | `io::Error` | כשל טעינה לפני ה־pipeline | אין | כן, המרת I/O לחריגת Python |
| D003 | Registry:45; API:101 | אותו סוג כשל ב־`dynamic_roles.json` | `io::Error` | כשל טעינה; אין fallback לרישום מובנה בלבד | אין | כן, המרת I/O |
| D004 | Registry:49–52,69–71 | JSON פגום או מבנה רישום לא מתאים: שדות נדרשים חסרים, טיפוסים לא מתאימים וכדומה | שגיאת serde_json בתוך `io::ErrorKind::InvalidData` | כשל טעינה של כל הרישום | אין | כן, `OSError` דרך המרת I/O |
| D005 | Loader:8–10; Runtime:41 | מקור חסר, נתיב לא תקין, הרשאות חסרות, תיקייה במקום קובץ או כשל מערכת קבצים אחר | `io::Error` → `RuntimeError::Load` | כשל טעינה לפני Tokenizer | אין | כן, `OSError` או תת־סוג כגון `FileNotFoundError`; תלוי בכשל ובמערכת |
| D006 | Loader:10; Runtime:41 | תוכן המקור אינו UTF-8 תקין | `io::Error` מסוג InvalidData → `Load` | כשל טעינה לפני Tokenizer | אין | כן, `OSError` |
| D007 | Scanner:25–28; Tokenizer:10; API:126–131 | הצהרת `@role` מזוהה ללא שם, לרבות שם שהוסר כהערה | `MissingRoleName { line }` → `RuntimeError::Tokenize` | הטוקניזציה כולה נעצרת; אין dispatch או מסירה | אין | כן, `ValueError` עם נתיב, שורה ו־`missing Role name` |
| D008 | Scanner:38–41; Tokenizer:10; API:126–131 | תוכן לא ריק ולא הערה לפני ה־Role הראשון | `ContentBeforeRole { line }` → `Tokenize` | הטוקניזציה כולה נעצרת | אין | כן, `ValueError` עם `content before first Role` |

### 3.2 רישום, ניתוב ומדיניות Runtime

| ID | רכיב ומיקום | תנאי | ייצוג נוכחי | השפעה נוכחית | פלט נוכחי | מגיע למשתמש? |
|---|---|---|---|---|---|---|
| D009 | Registry:56–64; Dispatcher:31 | שם לא נמצא בשני הרישומים | `LookupResult::Unknown` → `DispatchResult::Unknown { role }` | Dispatcher ממשיך; Runtime מדלג על המסירה של המופע | דרך D045 אם מגיעים אליו | כן, `RoleInfo.status="unknown"`; אין מופע חי |
| D010 | Registry:58–62; Dispatcher:32–43 | אותו שם נמצא בשני הרישומים | `LookupResult::Conflict` → `DispatchResult::Conflict` עם שני entries | אין הכרעה לפי קדימות; Dispatcher ממשיך לאסוף תוצאות | דרך D046 | כן, מטא־נתונים עם status ושני היעדים |
| D011 | Runtime:48–60 | לפחות תוצאת Conflict אחת בכל אוסף ה־dispatch | `has_conflict`; החזרת `Ok(LoadResult)` עם `delivered=[]` | כל שלב המסירה מבוטל לפני טעינת יעדים, גם עבור Resolved מוקדמים | כל ההתנגשויות ואז D047, כל עוד הכתיבה מצליחה | כן, `Project` ללא live Roles; Conflict עצמו אינו חריגה |
| D012 | Debug:10–38; Runtime:52,56,66; API:109 | כתיבת אחד מדיווחי Core נכשלת | `io::Error` → `RuntimeError::DebugOutput` | כשל טעינה מיידי; ייתכן פלט חלקי או מסירות קודמות | אין דיווח חלופי | כן, חריגת I/O, למשל BrokenPipe בהתאם לשגיאה |
| D013 | Bridges:57–58; Handoff:19–21 | `via` אינו מזהה Bridge רשום, לרבות ריק, `rust`, `Python` או רווח נוסף | `None` → `HandoffError::UnknownBridge(String)` | כשל המסירה הנוכחית והפסקת שאר הטעינה; אין ניחוש לפי סיומת | אין דיווח שגיאה ישיר; debug של Resolved כבר נכתב | כן, `RuntimeError` עם `UnknownBridge` |
| D014 | Handoff:22–24; Runtime:69–74; API:110–124 | Bridge מחזיר כשל מסירה | `Delivery(BridgeError)` → `RuntimeError::Handoff { name,index,target,error }` | עצירה בכשל הראשון, ללא rollback של השפעות קודמות | אין דיווח שגיאה ישיר | כן, `RuntimeError` עם שם, אינדקס גלובלי, יעד וקטגוריה |
| D015 | Bridges:38–41,48–53 | רישום Bridge תחת מזהה שכבר קיים | `HashMap::insert` מחזיר את הרישום הישן ב־`Option` | החלפה מפורשת מותרת פנימית; לא נוצר Conflict של Role | אין | לא דרך API ציבורי; הבונה הנוכחי רושם פעם אחת במפה ריקה |

### 3.3 Python Bridge וחוזה הקבלה

כל שגיאת `BridgeError` בטבלה זו עוברת דרך D014 ל־`RuntimeError` ציבורי. אף אחת מנקודות השגיאה הללו אינה מדפיסה הודעת שגיאה בעצמה.

| ID | רכיב ומיקום | תנאי | ייצוג נוכחי | השפעה נוכחית | פלט נוכחי | מגיע למשתמש? |
|---|---|---|---|---|---|---|
| D016 | PythonBridge:52–54 | canonicalize של יעד נכשל: קובץ חסר, נתיב או מערכת קבצים | `PythonTargetLoadFailure(error.to_string())` | עצירה לפני יצירת מודול | אין | כן, RuntimeError; לא חריגת I/O מקורית |
| D017 | PythonBridge:55–57 | נתיב קנוני שאינו ניתן לייצוג Unicode | `PythonTargetLoadFailure("target is not Unicode")` | עצירה לפני טעינת מודול | אין | כן, RuntimeError |
| D018 | PythonBridge:65–66 | ייבוא importlib.machinery או importlib.util נכשל | `PythonTargetLoadFailure(String)` | עצירת טעינה | אין | כן, RuntimeError |
| D019 | PythonBridge:67–70 | `SourceFileLoader` חסר או יצירתו נכשלת | `PythonTargetLoadFailure(String)` | עצירת טעינה | אין | כן, RuntimeError |
| D020 | PythonBridge:71–76 | כשל ב־`spec_from_loader` או `module_from_spec` | `PythonTargetLoadFailure(String)` | עצירת טעינה; אין ענף מיוחד ל־spec שאינו תקין | אין | כן, RuntimeError |
| D021 | PythonBridge:77–82 | ייבוא sys, גישה ל־modules או cast ל־PyDict נכשל | `PyErr` או שגיאת cast → `PythonTargetLoadFailure` | עצירה לפני התקנת המודול הזמני | אין | כן, RuntimeError |
| D022 | PythonBridge:83–84 | קריאת binding קודם או התקנת binding זמני ב־sys.modules נכשלת | `PythonTargetLoadFailure(String)` | יציאה לפני בלוק הטעינה והניקוי הרגיל | אין | כן, RuntimeError |
| D023 | PythonBridge:87–92 | קריאת מקור Python, פענוחו או קומפילציה נכשלת | `PythonTargetLoadFailure(String)` | ניסיון ניקוי binding ואז עצירה | אין | כן, לרבות SyntaxError/שגיאת קידוד שהפכו לפרט טקסטואלי |
| D024 | PythonBridge:93–96 | גישה ל־globals, ייבוא builtins או exec של קוד המודול נכשל | `PythonTargetLoadFailure(String)` | ניסיון ניקוי ואז עצירה; השפעות import שכבר התרחשו נשארות | רק פלט אפשרי של קוד היעד | כן, RuntimeError |
| D025 | PythonBridge:97–103 | `roleforge_receive` חסר או lookup שלו מעלה AttributeError | `MissingReceiver`; חריגה אחרת ב־lookup הופכת ל־`PythonTargetLoadFailure` | ניסיון ניקוי ואז עצירה; אין ניסיון שם חלופי | אין | כן, RuntimeError |
| D026 | PythonBridge:104–105 | receiver קיים אך אינו callable | `ReceiverNotCallable` | אין קריאה ל־receiver; ניסיון ניקוי ועצירה | אין | כן, RuntimeError |
| D027 | PythonBridge:107–108 | יצירת אובייקט PythonInput נכשלה | `InputConversion(String)` | אין יצירת live Role; ניסיון ניקוי ועצירה | אין | כן, RuntimeError |
| D028 | PythonBridge:109–112; Live:49–53 | ייבוא adapter או יצירת Role נכשלים; `Role` אינו מחלקה/יורש נדרש; בנאי מעלה חריגה | `TypeError("Target Role must subclass roleforge.Role")` או PyErr אחר → `InputConversion(String)` | receiver לא נקרא; ניסיון ניקוי ועצירה | אין, למעט פלט אפשרי מבנאי | כן, RuntimeError ולא TypeError המקורי |
| D029 | PythonBridge:113–115 | receiver מעלה חריגה, לרבות חתימה שאינה מקבלת את הארגומנט | `ReceiverRaised(String)` | ניסיון ניקוי ועצירת יתר המסירות; אין rollback | רק פלט אפשרי של receiver לפני הכשל | כן, RuntimeError |
| D030 | PythonBridge:118–123 | שחזור binding קודם או מחיקת binding זמני נכשלו אחרי פעולה מוצלחת | שגיאת cleanup → `PythonTargetLoadFailure(String)` | כשל טעינה גם אם receiver כבר סיים בהצלחה | אין | כן, RuntimeError; אם הפעולה נכשלה קודם, ראו D052 |

### 3.4 גבול Python והגישה הציבורית

| ID | רכיב ומיקום | תנאי | ייצוג נוכחי | השפעה נוכחית | פלט נוכחי | מגיע למשתמש? |
|---|---|---|---|---|---|---|
| D031 | Init:2–3; API:157–160 | כשל ייבוא ההרחבה/Live או רישום פונקציה ומחלקות במודול PyO3 | חריגת import או `PyErr` דרך `?` | כשל ייבוא החבילה/אתחול המודול | אין | כן, חריגת Python; סוגה תלוי במקור |
| D032 | API:90–91 | ארגומנטים לא מתאימים ל־load, או כשל המרת path ל־PathBuf | שגיאת binding/extraction של PyO3 | הקריאה נעצרת לפני גוף load | אין | כן, לרוב TypeError; חריגת פרוטוקול path יכולה להתפשט |
| D033 | API:79–85 | name לא מתאים ל־str, role_index לא מתאים ל־isize, או מספר מחוץ לטווח | שגיאת binding של PyO3 | קריאת הגישה בלבד נכשלת לפני Live | אין | כן, TypeError/OverflowError או חריגת המרה |
| D034 | API:134–143; Live:57–67 | כשל ייבוא `_live`, יצירת PyList או בניית `_ProjectRoles` | `PyErr` דרך `?` | load נכשל אחרי מסירות; אין rollback | אין ייעודי | כן, החריגה המקורית מגבול זה |
| D035 | API:144–151 | יצירת RoleInfo או tuple המטא־נתונים נכשלה | `PyResult`/`PyErr` דרך `?` | load נכשל אחרי מסירות ובניית קבוצות | אין | כן, חריגת יצירת אובייקט Python |
| D036 | Live:69–74; API:84–85 | אין קבוצה שנמסרה בהצלחה בשם המדויק | KeyError פנימי → KeyError עם `No successfully delivered Role named ...`, ללא שרשור מוצג | רק קריאת get_role נכשלת | אין | כן, KeyError; כולל שם Unknown או Resolved שלא נמסר עקב Conflict |
| D037 | Live:6–7,45–46 | אינדקס שאינו תומך בפרוטוקול integer; למשל str, float או slice | `operator.index` מעלה חריגה | רק פעולת הגישה נכשלת | אין | כן, TypeError בדוגמאות הרגילות; חריגה מותאמת מ־`__index__` אינה נתפסת |
| D038 | Live:8–11 | אינדקס מספרי שאינו מפתח של מופע חי, לרבות שלילי | KeyError → `IndexError("No live Role occurrence at role_index ...")` | רק פעולת הגישה נכשלת; שלילי אינו offset מהסוף | אין | כן, IndexError ללא שרשור מוצג |
| D039 | Live:76–79; API:79–80 | שם attribute אינו ממופה ל־Role חי | `AttributeError("No live Role attribute ...")` | רק גישת attribute נכשלת | אין | כן; `hasattr` עשוי להחזיר False |
| D040 | Live:80–83 | כמה שמות מדויקים ממופים לאותו lowercase attribute | `AttributeError("Ambiguous Role attribute ...; use get_role(exact_name)")` | אין בחירת מנצח; גישה בשם מדויק עדיין אפשרית | אין | כן, AttributeError |
| D041 | Live:65–67 | lowercase של שם Role אינו identifier או שהוא keyword | אי־הוספה ל־`attributes` | Role נשאר ב־groups; גישה בשם מדויק אפשרית | אין | בעקיפין: אין קיצור attribute; אין חריגה בזמן הבנייה |
| D042 | API:69–85; Live:65–67 | שם Role מתנגש ב־member קיים של Project, למשל roles/path/get_role | attribute lookup רגיל מוצא את ה־member לפני `__getattr__` | member קיים מקבל קדימות; live Role נגיש בשם מדויק | אין | כן, מוחזר ה־member הקיים; אין התרעה ייעודית |
| D043 | API:21,68–75; PythonBridge:10,24; Live:21–43 | ניסיון לשנות metadata לקריאה בלבד, או לגשת ל־member חסר באובייקט רגיל | descriptors/frozen PyO3 או Python מייצרים AttributeError | הפעולה המבוקשת בלבד נכשלת | אין | כן; אם זה קורה בתוך receiver, יומר בהמשך ל־D029 |

### 3.5 פלט, fallback והשמטת מידע

| ID | רכיב ומיקום | תנאי/אירוע | ייצוג נוכחי | השפעה נוכחית | פלט נוכחי | מגיע למשתמש? |
|---|---|---|---|---|---|---|
| D044 | Debug:10–19; Runtime:66 | Role נפתר, לפני מסירתו | `writeln!` על DispatchResult | ממשיכים למסירה אם הכתיבה מצליחה | `[CORE DEBUG]`, שם, שני אינדקסים, שורה, גוף ויעד | כן, stdout ב־load הציבורי; זה אינו אישור מסירה |
| D045 | Debug:20–22; Runtime:65–67 | Unknown במסלול ללא Conflict ולפני כשל אחר | `writeln!` | מדלגים על Role זה וממשיכים | `[RoleForge] Unknown Role: {name}` | כן; זיהוי המקור ב־D009 |
| D046 | Debug:23–33; Runtime:49–53 | Conflict במהלך preflight | `writeln!` | ממשיכים לדווח התנגשויות נוספות אם הכתיבה מצליחה | שם Role ושני נתיבי היעד | כן; אין הדפסת האינדקסים או via |
| D047 | Debug:37–38; Runtime:55–60 | preflight מצא Conflict | `writeln!` | חוזרים עם מטא־נתונים וללא מסירות | `[RoleForge] Handoff aborted.` | כן; כשל כתיבה משנה את התוצאה לחריגה |
| D048 | TestRole:7,13–19 | קבלת Test Role או קריאת hello | שמונה קריאות print בקוד הדוגמה הארוז | receiver מאתחל calls; hello מגדיל אותו ומחזיר body | כותרת, אישור קבלה, שם, אינדקסים, body, שורה; hello מדפיס ברכה | כן; כשל print בזמן receiver הופך D029, וב־hello מתפשט ישירות |
| D049 | Example:8–13 | load חזר בדוגמת הצרכן | print יחיד של הדוגמה וקריאות hello | המשך דוגמת השימוש; אין טיפול מקומי בשגיאות | `Loaded ... Roles from ...` ובהמשך פלט D048 | כן; המספר הוא מספר פריטי discovery, לא מספר מסירות |
| D050 | Live:49–50 | מודול יעד אינו מגדיר `Role` | `vars(module).get("Role", Role)` | יצירת Role בסיסי והמשך קבלה רגיל | אין | נגיש כאובייקט חי; fallback מכוון בחוזה |
| D051 | PythonBridge:113–116 | receiver מחזיר ערך בהצלחה | ערך `call1` נזרק; מוחזר אותו live object | אין משמעות פרוטוקולית לערך ההחזרה | אין | הערך אינו מגיע למשתמש דרך load; התעלמות מכוונת, לא כשל מדוכא |
| D052 | PythonBridge:118–123 | גם פעולת הטעינה/קבלה וגם cleanup נכשלו | cleanup חושב, אך `result?` מחזיר קודם את הכשל הראשי | הכשל הראשי נשמר, הכשל המשני אינו מדווח | אין | רק הכשל הראשי מגיע למשתמש |
| D053 | PythonBridge:51,54,108,112,115; API:117–124 | המרת חריגה למחרוזת ואז לשגיאה ציבורית חדשה | `PyErr.to_string()`/`io::Error.to_string()` → enum → Debug string → RuntimeError | אובדן אובייקט החריגה המקורי וה־traceback/שרשור המקוריים | אין ייעודי | טקסט וקטגוריה מגיעים; המבנה המקורי אינו נשמר |
| D054 | PythonBridge:60–63 | `write!` לתוך String לצורך שם מודול משתמש ב־unwrap | `Result` של formatting עובר unwrap | אין מסלול Err רגיל בכתיבת byte הקסדצימלי ל־String | אין | אינו כשל שניתן להפעיל בקלט רגיל; פירוט בסעיף 8 |

## 4. מסלולים מרכזיים: זיהוי, הפצה, מדיניות והצגה

### 4.1 Unknown Role

הזיהוי נמצא ב־`Registry::get_entry`: חיפוש שם מדויק בשתי המפות מחזיר `Unknown` אם אינו קיים באף אחת. ברגע זה זמינים השם והרישומים, אך ה־variant עצמו אינו נושא נתונים. Dispatcher מצרף את ה־`CleanRole` המקורי: שם, אינדקס גלובלי, אינדקס מקומי, גוף ושורת הצהרה.

Runtime מחזיק בכל תוצאות ה־dispatch. במסלול ללא Conflict הוא מעביר כל תוצאה ל־Debug, ואז קורא ל־Handoff רק עבור Resolved. לפיכך Unknown אינו מקבל Bridge או מופע חי, אינו משנה אינדקסים ואינו עוצר Roles אחרים. הוא אינו חריגה במהלך load. ב־Python הוא הופך ל־`RoleInfo` עם `status="unknown"` ונתיבי entry ריקים.

יש שתי הסתייגויות התלויות בזרימת הבקרה: כאשר יש Conflict כלשהו, לולאת המסירה כלל אינה מתחילה ולכן Unknown אינו מודפס, אף שהמטא־נתונים שלו מוחזרים. כאשר כשל קודם במסירה או בפלט כבר עצר את הטעינה, Unknown מאוחר יותר אינו מדווח ולא מוחזר Project. כשל בכתיבת הודעת Unknown עצמה כן מפיל את load דרך D012.

גישה מאוחרת לשם Unknown באמצעות `get_role` מעלה KeyError; גישת attribute מעלה AttributeError. אלו כשלים חדשים של גישה למופע חסר, ולא המרת Unknown לחריגה במהלך הטעינה.

### 4.2 Registry Conflict ו־preflight

הזיהוי נמצא ב־`Registry::get_entry`, כאשר שני החיפושים מצליחים. מתקבל `LookupResult::Conflict` עם השם ושני `RoleEntry`, שכל אחד כולל `via` ו־`target`. גם אם היעדים זהים, נוכחות בשני הרישומים מספיקה ל־Conflict. Dispatcher משמר את שתי הרשומות ומצרף את כל נתוני ה־Role. אין בחירה של built-in או dynamic ואין חריגה בשכבת Registry או Dispatcher.

Runtime עובר על אוסף התוצאות לפני מסירה כלשהי, מדפיס כל Conflict ומסמן `has_conflict`. אם נמצא אחד, הוא מדפיס ביטול ומחזיר `Ok` עם כל ה־discovery וללא delivered. שאר Roles נותחו לצורך ניתוב, אך אף אחד מהם אינו נמסר ואין טעינת מודול יעד, גם אם Conflict נמצא בסוף המקור.

ב־Python מתקבל `Project`; RoleInfo של Conflict כולל `builtin_entry` ו־`dynamic_entry`, ו־`entry` ריק. גם RoleInfo עם `status="resolved"` יכול להופיע באותו Project ללא מופע חי. `via` אינו נחשף בשדות RoleInfo. הפלט מציג שם ושני יעדים בלבד, ולכן אינו מבחין היטב בין מופעים חוזרים של אותה התנגשות.

ההתחייבות „כל ההתנגשויות מדווחות” מותנית ביכולת לכתוב פלט: D012 יכול לעצור כבר בהודעה הראשונה. רק אז load נכשל בחריגה; Conflict עצמו אינו מומר לשגיאה בהמשך.

בדיקת Conflict נעשית עבור שמות שנמצאו במקור, לא בסריקה כוללת של כל שמות Registry. התנגשות עבור שם שאינו משמש בקובץ לא תעצור את טעינתו.

### 4.3 Loader ותחביר המקור

Loader קורא `fs::read_to_string` ללא בדיקת סיומת `.rfg`, ללא fallback לנתיב אחר וללא טיפול נפרד בכל סוג כשל. הוא משמר `io::Error`; Runtime מוסיף את variant בשם Load, וה־API ממיר דרך `PyErr::from`. אין הוספת נתיב מקור מפורשת להודעת I/O בנקודות אלו, בשונה מהמרת שגיאת Tokenizer.

קובצי Registry נקראים עוד לפני Loader של המקור. לכן כשל במשאבי החבילה עשוי להופיע לפני בדיקה אם קובץ המקור קיים. JSON לא תקין ומבנה לא תקין חולקים InvalidData; שם קובץ Registry אינו מצורף במפורש בעת ההמרה.

ב־Tokenizer קיימים בדיוק שני variants של שגיאה: MissingRoleName ו־ContentBeforeRole. שגיאה מאוחרת מפסיקה את הטוקניזציה כולה; אין החזרת Roles חלקית ואין מסירה של Roles תקינים שנמצאו קודם.

| קלט/מצב | ההתנהגות בפועל |
|---|---|
| `@role`, או `@role # comment` | MissingRoleName |
| טקסט, או `    @role X`, לפני ההצהרה הראשונה | ContentBeforeRole |
| `    @role X` בתוך Role | תוכן אטום של ה־Role, ללא שגיאה |
| `@roles X`, `@end`, או `@role#comment` | אינם הצהרה מזוהה; לפני Role ייכשלו כ־ContentBeforeRole, ובתוכו יישמרו כגוף |
| שורות ריקות והערות מלאות לפני Role | מותרות |
| הערות מלאות בגוף | התוכן מוסר תוך שימור מבנה ירידות שורה |
| `#` בתוך שורת גוף | נשמר; אינו מנגנון הערות inline של Core |
| מקור ריק או הערות בלבד | הצלחה עם אפס Roles וללא פלט Core |
| שמות חוזרים, גוף ריק, שם חריג או מרובה מילים | אין בדיקת תקינות נוספת מעבר לשם שאינו ריק |

אין במימוש שגיאות עצמאיות כגון InvalidIndentation, InvalidRoleName או CommentNotAllowed. אין פרשנות ל־DSL הפנימי של Role. הסרת הערות וטיפול בקלט ריק הם עיבוד תחביר מכוון, לא דיכוי כשל.

### 4.4 Bridge, receiver והשלכות של כשל

Handoff פותר `via` בהתאמה מדויקת. רק `python` רשום בבנייה הציבורית הנוכחית. רישום ב־Registry עם יעד לא קיים או Bridge לא מוכר יכול עדיין להיחשב Resolved, כי אימות המסירה נעשה מאוחר יותר. אין preflight כולל של תקינות Bridges, קיום קבצים, מחלקות או receivers.

PythonBridge מבצע canonicalize, בונה שם מודול מנתיב מלא, טוען מקור, מקמפל ומבצע אותו. הוא מתקין binding זמני ב־sys.modules ומשחזר את הקודם או מוחק את הזמני לאחר פעולת הקבלה. כשל יכול להתרחש לפני התקנת binding, בתוך הפעולה, או בזמן ניקויו; הטבלה מבחינה ביניהם.

בדיקת receiver קודמת ליצירת ה־Role. `is_callable()` אינו מאמת חתימה; פונקציה עם חתימה לא תואמת תיכשל בקריאה ותסווג כ־ReceiverRaised. אם lookup מפעיל `__getattr__` של המודול שמעלה AttributeError מתוך לוגיקה פנימית, גם הוא יסווג MissingReceiver. אין בדיקה סטטית המבחינה בכך.

אם אין `Role` במילון המודול, נוצרת מחלקת הבסיס. אם קיים ערך לא תקין, אין fallback: נזרק TypeError שהופך InputConversion. אימות ירושה אינו מוכיח שהבנאי שמר `_input`, שהחזיר מופע תקין או שה־receiver שמר את מצב האובייקט. כשלים מאוחרים בגישה ל־name, role_index או בהצבת `_instances` עשויים להתגלות רק בבניית `_ProjectRoles` אחרי מסירות, ולעלות ישירות כחריגת Python דרך D034.

לא מבוצעת בדיקה מיוחדת ל־receiver אסינכרוני או generator. הקריאה נחשבת מוצלחת אם `call1` חזר בהצלחה; הערך נזרק ללא await או איטרציה. לכן callable כזה יכול לא לבצע את גוף הקבלה בפועל. אזהרה אפשרית של Python על coroutine שלא הומתן אינה אירוע שמזוהה או מוצג במנגנון RoleForge. זו מסקנה מהקוד, ולא התנהגות שנבדקה בהרצה כאן.

כשל מסירה עובר דרך Handoff ו־Runtime אל RuntimeError ציבורי. הוא כולל שם, אינדקס גלובלי ויעד, אך אינו כולל בשדות העטיפה את `role_index`, שורת ההצהרה או נתיב מקור ה־rfg. ה־receiver יכול לשנות קבצים, גלובלים או אובייקטים לפני הכשל; אין rollback. גם הצלחות קודמות אינן מוחזרות כ־Project חלקי.

### 4.5 Python API: metadata לעומת מופעים חיים

`project.roles` מכיל תוצאות גילוי וניתוב; `_ProjectRoles` נבנה רק מתוצאות שנמסרו. Unknown ו־Conflict אינם יוצרים live objects. גם Resolved אינו מוכיח מסירה כאשר ה־preflight ביטל הכול.

`get_role` מחפש שם מדויק ורגיש לאותיות. קיצור attribute מבוסס על `lower()` בלבד, ללא snake_case. שם שאינו identifier, keyword או שם שמתנגש ב־member קיים אינו מקבל גישה נוחה רגילה לאותו Role, אך ניתן להשתמש בשם המדויק. התנגשות lowercase בין שני Roles מעלה AttributeError בעת גישה, לא בזמן load.

בחירת מופע היא חיפוש במילון לפי role_index. שליליים אינם נספרים מהסוף; slices אינם נתמכים. `operator.index` מאפשר אובייקטים המממשים את פרוטוקול integer, ואין איסור מפורש על bool. יש הבדל בין `role[index]`, שעובר ישירות לפרוטוקול Python, לבין `get_role(name, index)`, שעובר קודם המרה ל־Rust isize ולכן מוגבל גם לטווח המספרי שלו. אם שם אינו קיים, חיפושו נכשל לפני `_select`; זהו סדר הבדיקות, ולא איחוד של כל שגיאות הגישה.

Live משתמש ב־`raise ... from None` בשתי המרות: KeyError פנימי לשגיאת שם ציבורית, ו־KeyError של אינדקס ל־IndexError. ההקשר המקורי אינו מוצג בשרשור החריגה הרגיל; זו הצגה מכוונת של שגיאת API.

לאחר load, קריאות למתודות של Role הן קריאות Python רגילות. חריגה מ־`hello()` או מכל מתודה אחרת שהמשתמש מפעיל אינה חוזרת למסלול Handoff ואינה עטופה אוטומטית ב־RuntimeError של RoleForge.

## 5. קיבוץ לפי אופי

הקבוצות חופפות במכוון, משום שגבול אחד עשוי גם להמיר כשל וגם להשמיט מידע. הן אינן דרגות חומרה.

| אופי | מזהים/נושאים |
|---|---|
| I/O ומשאבי חבילה | D001–D006, D012, D016–D017, חלק מ־D023 |
| תחביר מקור | D007–D008 |
| Registry ופתרון שמות | D009–D010, D013, D015 |
| Runtime ומדיניות pipeline | D011–D014, D030, D034–D035 |
| Bridge ו־interop | D016–D024, D027–D030, D031–D035 |
| חוזה Role | D025–D029, D043, D050–D051 |
| Python API ובחירת מופע | D032–D043 |
| פלט debug/מידע/דיווח תנאי | D044–D049 |
| fallback והשמטת מידע | D015, D041–D042, D050–D053; המרות from None ב־D036/D038 |
| הנחת תקינות פנימית | D054 |

## 6. מה נעצר ומה ממשיך

| מצב | היקף ההפסקה | מה נשמר או ממשיך |
|---|---|---|
| שגיאת ארגומנט, משאבי חבילה או Registry | load לפני ה־pipeline | אין מסירות ואין Project |
| שגיאת Loader או Tokenizer | כל load לפני dispatch/מסירה | אין תוצאה חלקית ואין פלט Core |
| Unknown בלבד | מסירת Role זה מדולגת | תוצאות discovery וכל מסירה אחרת ממשיכות, בכפוף לכשלים אחרים |
| Conflict כלשהו | כל שלב המסירה מבוטל | כל תוצאות discovery מוחזרות ב־Project אם הפלט ובניית Project מצליחים |
| Bridge לא מוכר או כשל Bridge | המסירה הנוכחית וכל המאוחרות | השפעות מסירות/ייבואים שכבר בוצעו אינן מתבטלות; אין Project |
| כשל פלט Core | כל load בנקודת הכתיבה | פלט חלקי והשפעות קודמות עלולים להישאר |
| כשל בניית מעטפת Python לאחר pipeline | החזרת Project נכשלת | מסירות שכבר בוצעו אינן מתבטלות |
| get_role/attribute/index לא תקינים | קריאת הגישה בלבד | Project ומופעים אחרים נשארים זמינים |
| שם ללא קיצור attribute, Role class חסרה, ערך receiver שנזרק | אין עצירת load בגלל מצב זה | נמשכת ההתנהגות החלופית המכוונת |

אין במימוש הנוכחי מדיניות כללית של „תפוס כשל receiver והמשך ל־Role הבא”. המצב שמאפשר דילוג מקומי מובנה הוא Unknown. אין גם שגיאת dispatch מסוג Result: Dispatcher מחזיר Vec של תוצאות מובנות לכל ה־Roles.

## 7. פלט ישיר ותנאים שקטים

### 7.1 כל נקודות הפלט הפעילות

בקוד Rust שאינו בדיקה יש **ארבע נקודות `writeln!`** ב־Debug: Resolved, Unknown, Conflict וביטול Handoff. ה־API הציבורי מחבר אותן ל־`io::stdout().lock()`. לא נמצאו `println!`, `eprintln!` או `dbg!` בקוד Rust הפעיל. `write!` ב־PythonBridge כותב ל־String, לא למסוף.

ב־Test Role הארוז יש **שמונה נקודות `print`**: שבע ב־receiver ואחת ב־hello. הן פלט של Role לדוגמה, ולא אבחון של Core. בדוגמת הצרכן יש **נקודת print אחת**, שאינה חלק מהספרייה. בסך הכול: 13 נקודות פלט מפורשות בקוד הפעיל ובדוגמה, ללא harness הבדיקות. D048 מאגד שמונה נקודות פלט לאותו פריט מלאי משום שהן אותה התנהגות דוגמה.

Debug הוא רכיב הצגה זמני נפרד, אך Runtime קורא לו כחלק הכרחי מהזרימה; אין תשתית Console/Event סופית. גם בחירת stdout נמצאת ישירות בגבול ה־API. לא נעשה כאן שינוי כדי לממש את הכיוון הארכיטקטוני העתידי.

### 7.2 מצבים שקטים או מידע שאינו נחשף

- Unknown אינו מודפס כאשר Conflict מבטל את שלב המסירה; הוא נשאר במטא־נתונים.
- רישום כפול עבור שם שלא מופיע במקור אינו נבדק על ידי preflight זה.
- Registry מפוענח ישירות ל־HashMap; אין בקוד בדיקה ייעודית לדיווח על מפתחות Role כפולים בתוך אותו מסמך JSON. אין לבלבל זאת עם Conflict בין שתי מפות. לא בוצע כאן probe של התנהגות parser במפתחות כפולים.
- Registry אינו מאמת מראש ש־via רשום או ש־target קיים. גם מחרוזת ריקה אינה מקבלת שגיאת ולידציה ייעודית בשכבת הרישום.
- D041–D042 אינם מפיקים התרעה על היעדר קיצור attribute או על הסתרתו.
- היעדר מחלקת Role מפעיל fallback מכוון; ערך החזרה מה־receiver נזרק בכוונה.
- כשל cleanup משני מוסתר כאשר הכשל הראשי כבר קיים.
- חריגות בתוך Bridge מאבדות את מבנה החריגה וה־traceback המקוריים בהמרה למחרוזת.
- `RoleInfo` אינו חושף via; עטיפת Handoff אינה משמרת את כל נתוני המקור הזמינים ב־CleanRole.
- בעת יצירת `_ProjectRoles`, כתיבה לאותו מפתח `(name, role_index)` הייתה דורסת ערך קיים ללא בדיקת כפילות. ה־Tokenizer מקצה מפתחות ייחודיים בזרימה הרגילה; אין בכך נקודת Conflict נוספת. מחלקת Role מותאמת עשויה לשנות התנהגות properties, ואין ולידציה נוספת בשכבה זו.
- מקור ריק, הערות, הופעות חוזרות של אותו שם וגופים ריקים הם קלטים מותרים, לא כשלים שהוסתרו.

## 8. Panic ונתיבי כשל פנימיים

נמצא **unwrap מפורש אחד בלבד בקוד production**, ב־PythonBridge:63. הוא עוטף formatting של byte ל־String. בהקשר זה String אינו sink שמחזיר שגיאת I/O, והפורמט הוא hex של byte; לא נמצא מסלול קלט רגיל שהופך אותו ל־Err. לפיכך אין בסיס לסווג אותו אוטומטית כבאג נגיש למשתמש. כשל הקצאה בתהליך הוא עניין נפרד ואינו מטופל במערכת שגיאות מקומית כאן.

לא נמצאו בקוד production קריאות מפורשות ל־`expect`, `panic!`, `unreachable!`, `todo!` או `unimplemented!`. זו אינה הוכחה שאין אפשרות panic בתלויות או כשלים ברמת סביבת הריצה. למשל `Python::initialize()` נקרא ללא ערוץ Result מקומי; הדוח אינו ממציא עבורו אירוע Core שאינו מוגדר בקוד.

Scanner חותך מחרוזות לפי offsets המחושבים מאורכי שורות שהתקבלו מהמקור עצמו. לא נמצא כאן חיתוך המבוסס על אינדקס משתמש חיצוני, ולכן אין בסיס לסמן שגיאת UTF-8 slicing כנתיב קלט רגיל. אין מערך של הודעות panic למקרי תחביר; שני הכשלים המוצהרים חוזרים כ־TokenizeError.

### בדיקות בלבד, מחוץ למלאי runtime

| מקום | מנגנונים שנמצאו | תפקיד |
|---|---|---|
| `roleforge/src/core/bridges/tests.rs` | unwrap ו־assert | אימות רישום, החלפה וחוזה מסירה |
| `roleforge/src/core/registry/tests.rs` | unwrap ו־assert | fixtures ואימות רישום |
| `roleforge/src/core/pipeline/dispatcher/tests.rs` | unwrap ו־panic בענף תוצאה בלתי צפוי | כישלון assertion מבני בבדיקה |
| `roleforge/src/core/pipeline/tokenizer/tokenizer/tests.rs` | unwrap ו־assert | אימות תחביר ומטא־נתונים |
| `roleforge/src/core/pipeline/loader/tests.rs` | unwrap_err; ניסיון חוזר על AlreadyExists; התעלמות משגיאת remove_file ב־Drop | יצירת fixtures, ציפייה לכשל וניקוי best-effort |
| `roleforge/src/core/pipeline/runtime/tests.rs` | unwrap, panic, unreachable; ניסיון חוזר על AlreadyExists; remove_file שנזרק ב־Drop | תשתית בדיקה ואימות תוצאות; BrokenOutput מדמה כשל פלט |
| `external_test_project/test_*.py` | assert/assertRaises, חריגות מכוונות, try/finally, subprocess ופלט בדיקה | אימות API ויצירת receivers פגומים באופן מכוון |

בדיקות Python משנות זמנית את קובצי Registry של החבילה המיובאת ומשחזרות אותם באמצעות finally או addCleanup. ה־README שלהן דורש הרצה סדרתית. אלו מנגנוני harness, לא fallback או suppression של הספרייה. `print('RESULT=...')` ופלטים מתוך subprocess בבדיקות אינם נקודות פלט production נוספות.

## 9. גבולות האחריות והיכן הם מעורבים

| מסלול | זיהוי | הפצה | החלטת המשך | הצגה |
|---|---|---|---|---|
| קובץ מקור | fs דרך Loader | io::Error → Load → PyErr | `?` ב־Runtime מפסיק pipeline | Python מציג חריגה אם לא נתפסה |
| תחביר | Scanner | TokenizeError → RuntimeError | Scanner חוזר בכשל הראשון; Runtime מפסיק | API מנסח ValueError עם נתיב ושורה |
| Unknown | Registry | LookupResult → DispatchResult → RoleInfo | Runtime מוסר רק Resolved | Debug מדפיס; metadata נגיש ב־Project |
| Conflict | Registry | LookupResult → DispatchResult → RoleInfo | Runtime מזהה Conflict באוסף ומבטל את כל המסירות | Debug מציג יעדים וביטול |
| Bridge | resolve או PythonBridge | HandoffError → RuntimeError → RuntimeError של Python | `?` בכל שכבה; Runtime מפסיק בהתחלה של כשל | API מנסח הודעת חריגה; Python מציג |
| גישה למופע | Live / binding של PyO3 | חריגת Python דרך API | הקריאה המקומית נעצרת | Live מנסח את ההודעה יחד עם הזיהוי |
| פלט | `writeln!` ב־Debug | io::Error → DebugOutput → PyErr | Runtime מפסיק גם עקב כשל הצגה | אין ערוץ פלט חלופי |

Registry ו־Dispatcher כבר מפרידים זיהוי מהצגה ואינם מדפיסים. גם Scanner אינו מנסח הודעה למשתמש. לעומת זאת, Live מזהה מצב, בוחר חריגה ומנסח את הטקסט באותה פונקציה. PythonBridge מזהה ומסווג כשלים אך גם משטח את פרטיהם לטקסט. API מתרגם קטגוריות ומעצב הודעות ציבוריות.

Runtime מערב החלטת המשך והפעלת הצגה: preflight מדווח תוך כדי בדיקת האוסף, וכתיבת פלט היא תנאי להמשך מסירה. תנאי Conflict מזוהה תחילה ב־Registry, אך בדיקת נוכחותו והחלטת הביטול מרוכזות באותו בלוק Runtime. Scanner, Bridge ו־Live משלבים זיהוי מקומי עם early return; זהו תיאור של המדיניות הקיימת, ללא המלצה לשנותה בשלב זה.

## 10. דפוסים חוזרים וחוסר אחידות נוכחי

1. **עטיפות בכמה שכבות:** BridgeError → HandoffError → RuntimeError → חריגת Python. לעומת זאת, כשל Registry עובר ישירות מ־io::Error ל־Python, וכשל בניית Project נשאר PyErr.
2. **מחרוזות במקום מידע מקורי:** שימוש חוזר ב־`to_string`, הודעות format ו־Debug formatting של enum. השם המקורי של חריגת Python עשוי להופיע בטקסט, אך היא אינה נשמרת כאובייקט או cause.
3. **מדיניות הכשל הראשון:** `?` חוזר ב־Loader, Tokenizer, Bridge, Runtime ובניית Project. Conflict הוא חריג לדפוס: נעשה ניסיון להציג את כל ההתנגשויות ולהחזיר תוצאת discovery מוצלחת.
4. **מידע מקור לא אחיד:** Tokenizer מספק שורה וה־API מוסיף נתיב; Handoff מספק שם/אינדקס גלובלי/יעד; Unknown מדפיס שם בלבד; Conflict מדפיס שם ושני יעדים; Resolved debug מציג יותר מידע מכולם.
5. **מחרוזות ציבוריות חוזרות:** Live בונה KeyError, IndexError ו־AttributeError בכמה מקומות, עם שני שימושים ב־`from None`.
6. **פלט בכמה תחומים:** Core debug, Test Role והצרכן מדפיסים בנפרד. אין ניתוב משותף, והפלט אינו אוסף אירועים מובנה.
7. **Resolved מול delivered:** אותו status נשמר גם כש־Conflict מונע מסירה. הודעת Resolved מופיעה לפני ניסיון המסירה. אלה מצבי ניתוב, לא אישור שה־receiver הושלם.
8. **משמעות „load הצליח”:** Conflict יכול להחזיר Project ריק ממופעים חיים; כשל cleanup אחרי receiver מוצלח יכול להפיל load. אלו תוצאות ברורות בקוד אך עלולות להפתיע משתמש.
9. **הקשר לחריגה משתנה לפי מיקום:** TypeError מבנאי Role מופיע כ־RuntimeError/InputConversion; TypeError בבניית `_ProjectRoles` נשאר חריגת Python ישירה; שגיאה במתודת Role לאחר load אינה עטופה כלל.
10. **אין preflight כולל לחוזה הקבלה:** רק Conflict מונע מראש את כל המסירות. יעד מאוחר פגום או Bridge לא מוכר מתגלים אחרי ש־Roles קודמים כבר עשויים להשפיע על הסביבה.
11. **אובדן כשל משני:** cleanup תמיד מנוסה לאחר בלוק הקבלה, אך תוצאתו אינה מדווחת כשהבלוק עצמו נכשל.
12. **כפילויות שונות מטופלות אחרת:** Conflict בין registries, החלפת Bridge פנימית, שמות Role חוזרים במקור, התנגשות lowercase וכפילויות מפתחות JSON אינם אותו מנגנון. אין להחיל על כולם סיווג יחיד.

הנקודות הללו מתעדות צרכים ושאלות לשלב הבא; הן אינן קובעות מנהלים, severity, קודים, event bus, logging framework או hierarchy חדשה של חריגות.

## 11. ראיות קיימות ומגבלות האימות

| נושא | ראיות בקוד הבדיקות הקיים |
|---|---|
| מקור חסר ו־UTF-8 שגוי | `loader/tests.rs`: missing_file_returns_not_found, invalid_utf8_returns_invalid_data; `test_python_api.py`: test_source_errors_cross_the_python_boundary |
| שני סוגי שגיאות תחביר וכללי הזחה/הערות | `tokenizer/tokenizer/tests.rs`: malformed_outer_source_returns_a_local_line_error, indented_declarations_before_first_role_are_content_errors, missing_names_including_comment_only_declarations_are_errors |
| Unknown ממשיך ושומר זהות | `dispatcher/tests.rs`: routes_in_source_order_and_continues_after_unknown_with_context; `test_handoff_preflight.py`: test_unknown_is_reported_and_later_resolved_role_is_processed |
| Conflict מאוחר מונע כל מסירה | `test_handoff_preflight.py`: test_late_conflict_prevents_processing_even_earlier_resolved_roles, test_all_conflicts_are_reported_before_aborting |
| כשל פלט הוא כשל Runtime | `runtime/tests.rs`: temporary_output_failure_is_returned |
| receiver חסר/לא callable/מעלה חריגה, יעד פגום ו־via לא מוכר | `test_handoff_preflight.py`: test_receiving_contract_failures_are_distinct, test_missing_target_fails, test_unknown_bridge_never_infers_python_from_extension |
| גישה, ambiguity, שמות שמורים ואינדקסים | `test_live_roles.py`: test_missing_and_invalid_access, test_case_collisions_reserved_names_and_unusual_names_have_exact_access |
| return נזרק ו־Role בסיסי נוצר | `test_live_roles.py`: test_legacy_receiver_without_class_retains_a_live_object_and_ignores_return |
| כשל receiver אינו מבטל השפעות קודמות | `test_live_roles.py`: test_receiver_failure_stops_later_delivery_without_rolling_back |
| מחלקת Role לא תקינה | `test_live_roles.py`: test_invalid_role_class_is_a_contextual_conversion_error |

המסלולים של כשל הקצאת אובייקטי Python, sys.modules שאינו dict, cleanup כפול, receiver אסינכרוני ובנאי שמשבש את חוזה האובייקט אינם מוכחים כאן על ידי הרצת בדיקה. תיאורם נגזר מהקריאות ומהענפים הקיימים. סוגים מדויקים של חריגות מערכת, שגיאות PyO3 אוטומטיות וניסוח הודעות תלויים גם בסביבת ההרצה; מקום שבו הקוד אינו מבטיח סוג מדויק מסומן בהתאם.

הספירה אינה כוללת כל אפשרות תאורטית בתוך Rust/Python או כל מתודה שרכיב צד שלישי עשוי להגדיר. היא מכסה את נקודות הזיהוי, ההמרה, ההפצה וההשמטה המשמעותיות שנמצאו במימוש RoleForge עצמו, לרבות גבולותיו לתלויות. לא נקבעה חומרה עתידית לאף תנאי.

## 12. מפת המצב הנוכחי

```text
רכיב
  ↓ מזהה או מקבל
תנאי / כשל / תוצאת ניתוב
  ↓ מיוצג כיום כ־
io::Error / TokenizeError / LookupResult / DispatchResult / BridgeError / PyErr
  ↓ עובר דרך
Dispatcher / Handoff / Runtime / Python API
  ↓ החלטת ההמשך הקיימת
Unknown: דילוג מקומי
Conflict: ביטול כל המסירות, החזרת discovery
כשל I/O / תחביר / מסירה / פלט: עצירה והחזרת חריגה
כשל גישה: עצירת הקריאה בלבד
  ↓ תוצאה למשתמש
פלט stdout זמני / RoleInfo / מופע חי / חריגת Python / מצב שקט
```

המסלול הזה מתאר את המערכת הקיימת בלבד. תכנון המערכת המרכזית נשאר לשלב הבא.
