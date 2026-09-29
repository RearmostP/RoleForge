# מיפוי המטא־דאטה של מערכת השגיאות

מסמך זה מסכם את הנתונים במימוש הנוכחי. מפת הכיסוי המלאה נמצאת ב־[דוח המימוש](../ERROR_SYSTEM_IMPLEMENTATION.md).
מזהי D001–D054 הם מזהי תצפיות מה[מלאי ההיסטורי](../../history/event_and_error_inventory.md), ולא קודי שגיאה או בהכרח סוגי שגיאה נפרדים.

| תצפיות | מקרים / ייצוג | נתונים בעלי משמעות |
|---|---|---|
| D001 | PackagePathResolutionFailed | requested_file בגבול Python; שם וסיווג קבועים על סוג החריגה |
| D002–D006 | FileNotFound, FileReadFailed, InvalidRegistry | נתיב המשאב; סוג I/O נשמר רק לצורך תאימות מחלקת החריגה |
| D007–D008 | MissingRoleName, ContentBeforeRole | שורת מקור; נתיב מתווסף בגבול Python |
| D009–D011, D044–D047 | UnknownRole, RoleConflict, ResolvedRole, HandoffAborted | מידע discovery קיים ושתי רשומות במקרה Conflict; ללא חומרה חדשה |
| D012 | OutputWriteFailed | סוג I/O לצורך תאימות חריגת הפלט בלבד |
| D013–D014, D016–D030 | UnknownBridge ומקרי Bridge/קבלה/ניקוי | מזהה Bridge כשאינו מוכר; שם Role, שני אינדקסים, שורה ויעד בהקשר מסירה |
| D034–D035 | ProjectConstructionFailed בהכנה ובהקצאות של Core; חריגות מקיבוץ מופעים חיים ממשיכות ללא עטיפה | requested_file בגבול Python |
| D036, D038–D040 | LiveRoleNotFound, RoleOccurrenceNotFound, LiveRoleAttributeNotFound, AmbiguousRoleAttribute | שם מדויק, אינדקס או attribute; שמות מתנגשים במקרה ambiguity |

אין איסוף חדש של סיבות, stack frames או שלבים פנימיים. שאר התצפיות מייצגות
התנהגות Python טבעית, fallback מאושר, בעלות של Role או מנגנוני הפצה/הצגה;
הן אינן יוצרות סוגי שגיאה נוספים. פירוט כל המזהים מופיע במסמך המימוש.

## תחזוקת המיפוי

בעת שינוי מקרה כשל יש לעדכן את הנתונים הנדרשים ואת מקורם לפי הקוד בפועל. מידע שכבר זמין מועבר לנקודת הדיווח; דמיון בשמות שדות אינו מצדיק מבנה משותף או מנגנון איסוף חדש.

[הרישום ההיסטורי וטבלת הסטטיסטיקה של D001](../../history/error_metadata_inventory.md) נשמרו בארכיון. הם אינם ספירה מלאה של המערכת הנוכחית.
