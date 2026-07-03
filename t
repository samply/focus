[1mdiff --git a/src/eucaim_sql.rs b/src/eucaim_sql.rs[m
[1mindex 543dcc5..32c56fb 100644[m
[1m--- a/src/eucaim_sql.rs[m
[1m+++ b/src/eucaim_sql.rs[m
[36m@@ -40,20 +40,20 @@[m [mpub static CRITERION: Lazy<HashMap<&str, &str>> = Lazy::new(|| {[m
     map.insert("SNOMEDCT363358000", "CLIN1000065"); // lung cancer[m
     map.insert("SNOMEDCT363484005", "CLIN1000087"); // pelvis cancer[m
     map.insert("SNOMEDCT399068003", "CLIN1000075"); // prostate cancer[m
[31m-    map.insert("RID10312", "MR"); // Modalities are integers in the DB! Search impossible until clarified![m
[31m-    map.insert("RID10337", "PET"); // Modalities are integers in the DB! Search impossible until clarified![m
[31m-    map.insert("RID10334", "SPECT"); // Modalities are integers in the DB! Search impossible until clarified![m
[31m-    map.insert("RID10321", "CT"); // Modalities are integers in the DB! Search impossible until clarified![m
[32m+[m[32m    map.insert("RID10312", "IMG1000038"); // MR[m
[32m+[m[32m    map.insert("RID10337", "IMG1004451"); // PET[m
[32m+[m[32m    map.insert("RID10334", "IMG1004450"); // SPECT[m
[32m+[m[32m    map.insert("RID10321", "IMG1000026"); // CT[m
     map.insert("SNOMEDCT76752008", "BP1000136");[m
     map.insert("SNOMEDCT71854001", "BP1000257");[m
     map.insert("SNOMEDCT39607008", "BP1000113");[m
     map.insert("SNOMEDCT12921003", "BP1000092");[m
     map.insert("SNOMEDCT41216001", "BP1000021");[m
[31m-    map.insert("C200140", "Siemens"); // can't find it in concepts[m
[31m-    map.insert("birnlex_3066", "Siemens"); // can't find it in concepts[m
[31m-    map.insert("birnlex_12833", "General%20Electric"); // can't find it in concepts[m
[31m-    map.insert("birnlex_3065", "Philips"); // can't find it in concepts[m
[31m-    map.insert("birnlex_3067", "Toshiba"); // can't find it in concepts[m
[32m+[m[32m    map.insert("C200140", "IMG1000044"); // Siemens[m
[32m+[m[32m    map.insert("birnlex_3066", "IMG1000044"); // Siemens[m
[32m+[m[32m    map.insert("birnlex_12833", "IMG1000047"); // GE[m
[32m+[m[32m    map.insert("birnlex_3065", "IMG1000046"); // Philips[m
[32m+[m[32m    map.insert("birnlex_3067", "IMG1000045"); // Toshiba[m
 [m
     map[m
 });[m
