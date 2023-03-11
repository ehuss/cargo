//! Tests for the `cargo doc` command with `-Zrustdoc-scrape-examples`.

use cargo_test_support::project;

#[test] fn basic0() { basic(); }
#[test] fn basic1() { basic(); }
#[test] fn basic2() { basic(); }
#[test] fn basic3() { basic(); }
#[test] fn basic4() { basic(); }
#[test] fn basic5() { basic(); }
#[test] fn basic6() { basic(); }
#[test] fn basic7() { basic(); }
#[test] fn basic8() { basic(); }
#[test] fn basic9() { basic(); }
#[test] fn basic10() { basic(); }
#[test] fn basic11() { basic(); }
#[test] fn basic12() { basic(); }
#[test] fn basic13() { basic(); }
#[test] fn basic14() { basic(); }
#[test] fn basic15() { basic(); }
#[test] fn basic16() { basic(); }
#[test] fn basic17() { basic(); }
#[test] fn basic18() { basic(); }
#[test] fn basic19() { basic(); }
#[test] fn basic20() { basic(); }
#[test] fn basic21() { basic(); }
#[test] fn basic22() { basic(); }
#[test] fn basic23() { basic(); }
#[test] fn basic24() { basic(); }
#[test] fn basic25() { basic(); }
#[test] fn basic26() { basic(); }
#[test] fn basic27() { basic(); }
#[test] fn basic28() { basic(); }
#[test] fn basic29() { basic(); }
#[test] fn basic30() { basic(); }
#[test] fn basic31() { basic(); }
#[test] fn basic32() { basic(); }
#[test] fn basic33() { basic(); }
#[test] fn basic34() { basic(); }
#[test] fn basic35() { basic(); }
#[test] fn basic36() { basic(); }
#[test] fn basic37() { basic(); }
#[test] fn basic38() { basic(); }
#[test] fn basic39() { basic(); }
#[test] fn basic40() { basic(); }
#[test] fn basic41() { basic(); }
#[test] fn basic42() { basic(); }
#[test] fn basic43() { basic(); }
#[test] fn basic44() { basic(); }
#[test] fn basic45() { basic(); }
#[test] fn basic46() { basic(); }
#[test] fn basic47() { basic(); }
#[test] fn basic48() { basic(); }
#[test] fn basic49() { basic(); }
#[test] fn basic50() { basic(); }
#[test] fn basic51() { basic(); }
#[test] fn basic52() { basic(); }
#[test] fn basic53() { basic(); }
#[test] fn basic54() { basic(); }
#[test] fn basic55() { basic(); }
#[test] fn basic56() { basic(); }
#[test] fn basic57() { basic(); }
#[test] fn basic58() { basic(); }
#[test] fn basic59() { basic(); }
#[test] fn basic60() { basic(); }
#[test] fn basic61() { basic(); }
#[test] fn basic62() { basic(); }
#[test] fn basic63() { basic(); }
#[test] fn basic64() { basic(); }
#[test] fn basic65() { basic(); }
#[test] fn basic66() { basic(); }
#[test] fn basic67() { basic(); }
#[test] fn basic68() { basic(); }
#[test] fn basic69() { basic(); }
#[test] fn basic70() { basic(); }
#[test] fn basic71() { basic(); }
#[test] fn basic72() { basic(); }
#[test] fn basic73() { basic(); }
#[test] fn basic74() { basic(); }
#[test] fn basic75() { basic(); }
#[test] fn basic76() { basic(); }
#[test] fn basic77() { basic(); }
#[test] fn basic78() { basic(); }
#[test] fn basic79() { basic(); }
#[test] fn basic80() { basic(); }
#[test] fn basic81() { basic(); }
#[test] fn basic82() { basic(); }
#[test] fn basic83() { basic(); }
#[test] fn basic84() { basic(); }
#[test] fn basic85() { basic(); }
#[test] fn basic86() { basic(); }
#[test] fn basic87() { basic(); }
#[test] fn basic88() { basic(); }
#[test] fn basic89() { basic(); }
#[test] fn basic90() { basic(); }
#[test] fn basic91() { basic(); }
#[test] fn basic92() { basic(); }
#[test] fn basic93() { basic(); }
#[test] fn basic94() { basic(); }
#[test] fn basic95() { basic(); }
#[test] fn basic96() { basic(); }
#[test] fn basic97() { basic(); }
#[test] fn basic98() { basic(); }
#[test] fn basic99() { basic(); }
#[test] fn basic100() { basic(); }
#[test] fn basic101() { basic(); }
#[test] fn basic102() { basic(); }
#[test] fn basic103() { basic(); }
#[test] fn basic104() { basic(); }
#[test] fn basic105() { basic(); }
#[test] fn basic106() { basic(); }
#[test] fn basic107() { basic(); }
#[test] fn basic108() { basic(); }
#[test] fn basic109() { basic(); }
#[test] fn basic110() { basic(); }
#[test] fn basic111() { basic(); }
#[test] fn basic112() { basic(); }
#[test] fn basic113() { basic(); }
#[test] fn basic114() { basic(); }
#[test] fn basic115() { basic(); }
#[test] fn basic116() { basic(); }
#[test] fn basic117() { basic(); }
#[test] fn basic118() { basic(); }
#[test] fn basic119() { basic(); }
#[test] fn basic120() { basic(); }
#[test] fn basic121() { basic(); }
#[test] fn basic122() { basic(); }
#[test] fn basic123() { basic(); }
#[test] fn basic124() { basic(); }
#[test] fn basic125() { basic(); }
#[test] fn basic126() { basic(); }
#[test] fn basic127() { basic(); }
#[test] fn basic128() { basic(); }
#[test] fn basic129() { basic(); }
#[test] fn basic130() { basic(); }
#[test] fn basic131() { basic(); }
#[test] fn basic132() { basic(); }
#[test] fn basic133() { basic(); }
#[test] fn basic134() { basic(); }
#[test] fn basic135() { basic(); }
#[test] fn basic136() { basic(); }
#[test] fn basic137() { basic(); }
#[test] fn basic138() { basic(); }
#[test] fn basic139() { basic(); }
#[test] fn basic140() { basic(); }
#[test] fn basic141() { basic(); }
#[test] fn basic142() { basic(); }
#[test] fn basic143() { basic(); }
#[test] fn basic144() { basic(); }
#[test] fn basic145() { basic(); }
#[test] fn basic146() { basic(); }
#[test] fn basic147() { basic(); }
#[test] fn basic148() { basic(); }
#[test] fn basic149() { basic(); }
#[test] fn basic150() { basic(); }
#[test] fn basic151() { basic(); }
#[test] fn basic152() { basic(); }
#[test] fn basic153() { basic(); }
#[test] fn basic154() { basic(); }
#[test] fn basic155() { basic(); }
#[test] fn basic156() { basic(); }
#[test] fn basic157() { basic(); }
#[test] fn basic158() { basic(); }
#[test] fn basic159() { basic(); }
#[test] fn basic160() { basic(); }
#[test] fn basic161() { basic(); }
#[test] fn basic162() { basic(); }
#[test] fn basic163() { basic(); }
#[test] fn basic164() { basic(); }
#[test] fn basic165() { basic(); }
#[test] fn basic166() { basic(); }
#[test] fn basic167() { basic(); }
#[test] fn basic168() { basic(); }
#[test] fn basic169() { basic(); }
#[test] fn basic170() { basic(); }
#[test] fn basic171() { basic(); }
#[test] fn basic172() { basic(); }
#[test] fn basic173() { basic(); }
#[test] fn basic174() { basic(); }
#[test] fn basic175() { basic(); }
#[test] fn basic176() { basic(); }
#[test] fn basic177() { basic(); }
#[test] fn basic178() { basic(); }
#[test] fn basic179() { basic(); }
#[test] fn basic180() { basic(); }
#[test] fn basic181() { basic(); }
#[test] fn basic182() { basic(); }
#[test] fn basic183() { basic(); }
#[test] fn basic184() { basic(); }
#[test] fn basic185() { basic(); }
#[test] fn basic186() { basic(); }
#[test] fn basic187() { basic(); }
#[test] fn basic188() { basic(); }
#[test] fn basic189() { basic(); }
#[test] fn basic190() { basic(); }
#[test] fn basic191() { basic(); }
#[test] fn basic192() { basic(); }
#[test] fn basic193() { basic(); }
#[test] fn basic194() { basic(); }
#[test] fn basic195() { basic(); }
#[test] fn basic196() { basic(); }
#[test] fn basic197() { basic(); }
#[test] fn basic198() { basic(); }
#[test] fn basic199() { basic(); }
#[test] fn basic200() { basic(); }
#[test] fn basic201() { basic(); }
#[test] fn basic202() { basic(); }
#[test] fn basic203() { basic(); }
#[test] fn basic204() { basic(); }
#[test] fn basic205() { basic(); }
#[test] fn basic206() { basic(); }
#[test] fn basic207() { basic(); }
#[test] fn basic208() { basic(); }
#[test] fn basic209() { basic(); }
#[test] fn basic210() { basic(); }
#[test] fn basic211() { basic(); }
#[test] fn basic212() { basic(); }
#[test] fn basic213() { basic(); }
#[test] fn basic214() { basic(); }
#[test] fn basic215() { basic(); }
#[test] fn basic216() { basic(); }
#[test] fn basic217() { basic(); }
#[test] fn basic218() { basic(); }
#[test] fn basic219() { basic(); }
#[test] fn basic220() { basic(); }
#[test] fn basic221() { basic(); }
#[test] fn basic222() { basic(); }
#[test] fn basic223() { basic(); }
#[test] fn basic224() { basic(); }
#[test] fn basic225() { basic(); }
#[test] fn basic226() { basic(); }
#[test] fn basic227() { basic(); }
#[test] fn basic228() { basic(); }
#[test] fn basic229() { basic(); }
#[test] fn basic230() { basic(); }
#[test] fn basic231() { basic(); }
#[test] fn basic232() { basic(); }
#[test] fn basic233() { basic(); }
#[test] fn basic234() { basic(); }
#[test] fn basic235() { basic(); }
#[test] fn basic236() { basic(); }
#[test] fn basic237() { basic(); }
#[test] fn basic238() { basic(); }
#[test] fn basic239() { basic(); }
#[test] fn basic240() { basic(); }
#[test] fn basic241() { basic(); }
#[test] fn basic242() { basic(); }
#[test] fn basic243() { basic(); }
#[test] fn basic244() { basic(); }
#[test] fn basic245() { basic(); }
#[test] fn basic246() { basic(); }
#[test] fn basic247() { basic(); }
#[test] fn basic248() { basic(); }
#[test] fn basic249() { basic(); }
#[test] fn basic250() { basic(); }
#[test] fn basic251() { basic(); }
#[test] fn basic252() { basic(); }
#[test] fn basic253() { basic(); }
#[test] fn basic254() { basic(); }
#[test] fn basic255() { basic(); }
#[test] fn basic256() { basic(); }
#[test] fn basic257() { basic(); }
#[test] fn basic258() { basic(); }
#[test] fn basic259() { basic(); }
#[test] fn basic260() { basic(); }
#[test] fn basic261() { basic(); }
#[test] fn basic262() { basic(); }
#[test] fn basic263() { basic(); }
#[test] fn basic264() { basic(); }
#[test] fn basic265() { basic(); }
#[test] fn basic266() { basic(); }
#[test] fn basic267() { basic(); }
#[test] fn basic268() { basic(); }
#[test] fn basic269() { basic(); }
#[test] fn basic270() { basic(); }
#[test] fn basic271() { basic(); }
#[test] fn basic272() { basic(); }
#[test] fn basic273() { basic(); }
#[test] fn basic274() { basic(); }
#[test] fn basic275() { basic(); }
#[test] fn basic276() { basic(); }
#[test] fn basic277() { basic(); }
#[test] fn basic278() { basic(); }
#[test] fn basic279() { basic(); }
#[test] fn basic280() { basic(); }
#[test] fn basic281() { basic(); }
#[test] fn basic282() { basic(); }
#[test] fn basic283() { basic(); }
#[test] fn basic284() { basic(); }
#[test] fn basic285() { basic(); }
#[test] fn basic286() { basic(); }
#[test] fn basic287() { basic(); }
#[test] fn basic288() { basic(); }
#[test] fn basic289() { basic(); }
#[test] fn basic290() { basic(); }
#[test] fn basic291() { basic(); }
#[test] fn basic292() { basic(); }
#[test] fn basic293() { basic(); }
#[test] fn basic294() { basic(); }
#[test] fn basic295() { basic(); }
#[test] fn basic296() { basic(); }
#[test] fn basic297() { basic(); }
#[test] fn basic298() { basic(); }
#[test] fn basic299() { basic(); }
#[test] fn basic300() { basic(); }
#[test] fn basic301() { basic(); }
#[test] fn basic302() { basic(); }
#[test] fn basic303() { basic(); }
#[test] fn basic304() { basic(); }
#[test] fn basic305() { basic(); }
#[test] fn basic306() { basic(); }
#[test] fn basic307() { basic(); }
#[test] fn basic308() { basic(); }
#[test] fn basic309() { basic(); }
#[test] fn basic310() { basic(); }
#[test] fn basic311() { basic(); }
#[test] fn basic312() { basic(); }
#[test] fn basic313() { basic(); }
#[test] fn basic314() { basic(); }
#[test] fn basic315() { basic(); }
#[test] fn basic316() { basic(); }
#[test] fn basic317() { basic(); }
#[test] fn basic318() { basic(); }
#[test] fn basic319() { basic(); }
#[test] fn basic320() { basic(); }
#[test] fn basic321() { basic(); }
#[test] fn basic322() { basic(); }
#[test] fn basic323() { basic(); }
#[test] fn basic324() { basic(); }
#[test] fn basic325() { basic(); }
#[test] fn basic326() { basic(); }
#[test] fn basic327() { basic(); }
#[test] fn basic328() { basic(); }
#[test] fn basic329() { basic(); }
#[test] fn basic330() { basic(); }
#[test] fn basic331() { basic(); }
#[test] fn basic332() { basic(); }
#[test] fn basic333() { basic(); }
#[test] fn basic334() { basic(); }
#[test] fn basic335() { basic(); }
#[test] fn basic336() { basic(); }
#[test] fn basic337() { basic(); }
#[test] fn basic338() { basic(); }
#[test] fn basic339() { basic(); }
#[test] fn basic340() { basic(); }
#[test] fn basic341() { basic(); }
#[test] fn basic342() { basic(); }
#[test] fn basic343() { basic(); }
#[test] fn basic344() { basic(); }
#[test] fn basic345() { basic(); }
#[test] fn basic346() { basic(); }
#[test] fn basic347() { basic(); }
#[test] fn basic348() { basic(); }
#[test] fn basic349() { basic(); }
#[test] fn basic350() { basic(); }
#[test] fn basic351() { basic(); }
#[test] fn basic352() { basic(); }
#[test] fn basic353() { basic(); }
#[test] fn basic354() { basic(); }
#[test] fn basic355() { basic(); }
#[test] fn basic356() { basic(); }
#[test] fn basic357() { basic(); }
#[test] fn basic358() { basic(); }
#[test] fn basic359() { basic(); }
#[test] fn basic360() { basic(); }
#[test] fn basic361() { basic(); }
#[test] fn basic362() { basic(); }
#[test] fn basic363() { basic(); }
#[test] fn basic364() { basic(); }
#[test] fn basic365() { basic(); }
#[test] fn basic366() { basic(); }
#[test] fn basic367() { basic(); }
#[test] fn basic368() { basic(); }
#[test] fn basic369() { basic(); }
#[test] fn basic370() { basic(); }
#[test] fn basic371() { basic(); }
#[test] fn basic372() { basic(); }
#[test] fn basic373() { basic(); }
#[test] fn basic374() { basic(); }
#[test] fn basic375() { basic(); }
#[test] fn basic376() { basic(); }
#[test] fn basic377() { basic(); }
#[test] fn basic378() { basic(); }
#[test] fn basic379() { basic(); }
#[test] fn basic380() { basic(); }
#[test] fn basic381() { basic(); }
#[test] fn basic382() { basic(); }
#[test] fn basic383() { basic(); }
#[test] fn basic384() { basic(); }
#[test] fn basic385() { basic(); }
#[test] fn basic386() { basic(); }
#[test] fn basic387() { basic(); }
#[test] fn basic388() { basic(); }
#[test] fn basic389() { basic(); }
#[test] fn basic390() { basic(); }
#[test] fn basic391() { basic(); }
#[test] fn basic392() { basic(); }
#[test] fn basic393() { basic(); }
#[test] fn basic394() { basic(); }
#[test] fn basic395() { basic(); }
#[test] fn basic396() { basic(); }
#[test] fn basic397() { basic(); }
#[test] fn basic398() { basic(); }
#[test] fn basic399() { basic(); }
#[test] fn basic400() { basic(); }
#[test] fn basic401() { basic(); }
#[test] fn basic402() { basic(); }
#[test] fn basic403() { basic(); }
#[test] fn basic404() { basic(); }
#[test] fn basic405() { basic(); }
#[test] fn basic406() { basic(); }
#[test] fn basic407() { basic(); }
#[test] fn basic408() { basic(); }
#[test] fn basic409() { basic(); }
#[test] fn basic410() { basic(); }
#[test] fn basic411() { basic(); }
#[test] fn basic412() { basic(); }
#[test] fn basic413() { basic(); }
#[test] fn basic414() { basic(); }
#[test] fn basic415() { basic(); }
#[test] fn basic416() { basic(); }
#[test] fn basic417() { basic(); }
#[test] fn basic418() { basic(); }
#[test] fn basic419() { basic(); }
#[test] fn basic420() { basic(); }
#[test] fn basic421() { basic(); }
#[test] fn basic422() { basic(); }
#[test] fn basic423() { basic(); }
#[test] fn basic424() { basic(); }
#[test] fn basic425() { basic(); }
#[test] fn basic426() { basic(); }
#[test] fn basic427() { basic(); }
#[test] fn basic428() { basic(); }
#[test] fn basic429() { basic(); }
#[test] fn basic430() { basic(); }
#[test] fn basic431() { basic(); }
#[test] fn basic432() { basic(); }
#[test] fn basic433() { basic(); }
#[test] fn basic434() { basic(); }
#[test] fn basic435() { basic(); }
#[test] fn basic436() { basic(); }
#[test] fn basic437() { basic(); }
#[test] fn basic438() { basic(); }
#[test] fn basic439() { basic(); }
#[test] fn basic440() { basic(); }
#[test] fn basic441() { basic(); }
#[test] fn basic442() { basic(); }
#[test] fn basic443() { basic(); }
#[test] fn basic444() { basic(); }
#[test] fn basic445() { basic(); }
#[test] fn basic446() { basic(); }
#[test] fn basic447() { basic(); }
#[test] fn basic448() { basic(); }
#[test] fn basic449() { basic(); }
#[test] fn basic450() { basic(); }
#[test] fn basic451() { basic(); }
#[test] fn basic452() { basic(); }
#[test] fn basic453() { basic(); }
#[test] fn basic454() { basic(); }
#[test] fn basic455() { basic(); }
#[test] fn basic456() { basic(); }
#[test] fn basic457() { basic(); }
#[test] fn basic458() { basic(); }
#[test] fn basic459() { basic(); }
#[test] fn basic460() { basic(); }
#[test] fn basic461() { basic(); }
#[test] fn basic462() { basic(); }
#[test] fn basic463() { basic(); }
#[test] fn basic464() { basic(); }
#[test] fn basic465() { basic(); }
#[test] fn basic466() { basic(); }
#[test] fn basic467() { basic(); }
#[test] fn basic468() { basic(); }
#[test] fn basic469() { basic(); }
#[test] fn basic470() { basic(); }
#[test] fn basic471() { basic(); }
#[test] fn basic472() { basic(); }
#[test] fn basic473() { basic(); }
#[test] fn basic474() { basic(); }
#[test] fn basic475() { basic(); }
#[test] fn basic476() { basic(); }
#[test] fn basic477() { basic(); }
#[test] fn basic478() { basic(); }
#[test] fn basic479() { basic(); }
#[test] fn basic480() { basic(); }
#[test] fn basic481() { basic(); }
#[test] fn basic482() { basic(); }
#[test] fn basic483() { basic(); }
#[test] fn basic484() { basic(); }
#[test] fn basic485() { basic(); }
#[test] fn basic486() { basic(); }
#[test] fn basic487() { basic(); }
#[test] fn basic488() { basic(); }
#[test] fn basic489() { basic(); }
#[test] fn basic490() { basic(); }
#[test] fn basic491() { basic(); }
#[test] fn basic492() { basic(); }
#[test] fn basic493() { basic(); }
#[test] fn basic494() { basic(); }
#[test] fn basic495() { basic(); }
#[test] fn basic496() { basic(); }
#[test] fn basic497() { basic(); }
#[test] fn basic498() { basic(); }
#[test] fn basic499() { basic(); }
#[test] fn basic500() { basic(); }
#[test] fn basic501() { basic(); }
#[test] fn basic502() { basic(); }
#[test] fn basic503() { basic(); }
#[test] fn basic504() { basic(); }
#[test] fn basic505() { basic(); }
#[test] fn basic506() { basic(); }
#[test] fn basic507() { basic(); }
#[test] fn basic508() { basic(); }
#[test] fn basic509() { basic(); }
#[test] fn basic510() { basic(); }
#[test] fn basic511() { basic(); }
#[test] fn basic512() { basic(); }
#[test] fn basic513() { basic(); }
#[test] fn basic514() { basic(); }
#[test] fn basic515() { basic(); }
#[test] fn basic516() { basic(); }
#[test] fn basic517() { basic(); }
#[test] fn basic518() { basic(); }
#[test] fn basic519() { basic(); }
#[test] fn basic520() { basic(); }
#[test] fn basic521() { basic(); }
#[test] fn basic522() { basic(); }
#[test] fn basic523() { basic(); }
#[test] fn basic524() { basic(); }
#[test] fn basic525() { basic(); }
#[test] fn basic526() { basic(); }
#[test] fn basic527() { basic(); }
#[test] fn basic528() { basic(); }
#[test] fn basic529() { basic(); }
#[test] fn basic530() { basic(); }
#[test] fn basic531() { basic(); }
#[test] fn basic532() { basic(); }
#[test] fn basic533() { basic(); }
#[test] fn basic534() { basic(); }
#[test] fn basic535() { basic(); }
#[test] fn basic536() { basic(); }
#[test] fn basic537() { basic(); }
#[test] fn basic538() { basic(); }
#[test] fn basic539() { basic(); }
#[test] fn basic540() { basic(); }
#[test] fn basic541() { basic(); }
#[test] fn basic542() { basic(); }
#[test] fn basic543() { basic(); }
#[test] fn basic544() { basic(); }
#[test] fn basic545() { basic(); }
#[test] fn basic546() { basic(); }
#[test] fn basic547() { basic(); }
#[test] fn basic548() { basic(); }
#[test] fn basic549() { basic(); }
#[test] fn basic550() { basic(); }
#[test] fn basic551() { basic(); }
#[test] fn basic552() { basic(); }
#[test] fn basic553() { basic(); }
#[test] fn basic554() { basic(); }
#[test] fn basic555() { basic(); }
#[test] fn basic556() { basic(); }
#[test] fn basic557() { basic(); }
#[test] fn basic558() { basic(); }
#[test] fn basic559() { basic(); }
#[test] fn basic560() { basic(); }
#[test] fn basic561() { basic(); }
#[test] fn basic562() { basic(); }
#[test] fn basic563() { basic(); }
#[test] fn basic564() { basic(); }
#[test] fn basic565() { basic(); }
#[test] fn basic566() { basic(); }
#[test] fn basic567() { basic(); }
#[test] fn basic568() { basic(); }
#[test] fn basic569() { basic(); }
#[test] fn basic570() { basic(); }
#[test] fn basic571() { basic(); }
#[test] fn basic572() { basic(); }
#[test] fn basic573() { basic(); }
#[test] fn basic574() { basic(); }
#[test] fn basic575() { basic(); }
#[test] fn basic576() { basic(); }
#[test] fn basic577() { basic(); }
#[test] fn basic578() { basic(); }
#[test] fn basic579() { basic(); }
#[test] fn basic580() { basic(); }
#[test] fn basic581() { basic(); }
#[test] fn basic582() { basic(); }
#[test] fn basic583() { basic(); }
#[test] fn basic584() { basic(); }
#[test] fn basic585() { basic(); }
#[test] fn basic586() { basic(); }
#[test] fn basic587() { basic(); }
#[test] fn basic588() { basic(); }
#[test] fn basic589() { basic(); }
#[test] fn basic590() { basic(); }
#[test] fn basic591() { basic(); }
#[test] fn basic592() { basic(); }
#[test] fn basic593() { basic(); }
#[test] fn basic594() { basic(); }
#[test] fn basic595() { basic(); }
#[test] fn basic596() { basic(); }
#[test] fn basic597() { basic(); }
#[test] fn basic598() { basic(); }
#[test] fn basic599() { basic(); }
#[test] fn basic600() { basic(); }
#[test] fn basic601() { basic(); }
#[test] fn basic602() { basic(); }
#[test] fn basic603() { basic(); }
#[test] fn basic604() { basic(); }
#[test] fn basic605() { basic(); }
#[test] fn basic606() { basic(); }
#[test] fn basic607() { basic(); }
#[test] fn basic608() { basic(); }
#[test] fn basic609() { basic(); }
#[test] fn basic610() { basic(); }
#[test] fn basic611() { basic(); }
#[test] fn basic612() { basic(); }
#[test] fn basic613() { basic(); }
#[test] fn basic614() { basic(); }
#[test] fn basic615() { basic(); }
#[test] fn basic616() { basic(); }
#[test] fn basic617() { basic(); }
#[test] fn basic618() { basic(); }
#[test] fn basic619() { basic(); }
#[test] fn basic620() { basic(); }
#[test] fn basic621() { basic(); }
#[test] fn basic622() { basic(); }
#[test] fn basic623() { basic(); }
#[test] fn basic624() { basic(); }
#[test] fn basic625() { basic(); }
#[test] fn basic626() { basic(); }
#[test] fn basic627() { basic(); }
#[test] fn basic628() { basic(); }
#[test] fn basic629() { basic(); }
#[test] fn basic630() { basic(); }
#[test] fn basic631() { basic(); }
#[test] fn basic632() { basic(); }
#[test] fn basic633() { basic(); }
#[test] fn basic634() { basic(); }
#[test] fn basic635() { basic(); }
#[test] fn basic636() { basic(); }
#[test] fn basic637() { basic(); }
#[test] fn basic638() { basic(); }
#[test] fn basic639() { basic(); }
#[test] fn basic640() { basic(); }
#[test] fn basic641() { basic(); }
#[test] fn basic642() { basic(); }
#[test] fn basic643() { basic(); }
#[test] fn basic644() { basic(); }
#[test] fn basic645() { basic(); }
#[test] fn basic646() { basic(); }
#[test] fn basic647() { basic(); }
#[test] fn basic648() { basic(); }
#[test] fn basic649() { basic(); }
#[test] fn basic650() { basic(); }
#[test] fn basic651() { basic(); }
#[test] fn basic652() { basic(); }
#[test] fn basic653() { basic(); }
#[test] fn basic654() { basic(); }
#[test] fn basic655() { basic(); }
#[test] fn basic656() { basic(); }
#[test] fn basic657() { basic(); }
#[test] fn basic658() { basic(); }
#[test] fn basic659() { basic(); }
#[test] fn basic660() { basic(); }
#[test] fn basic661() { basic(); }
#[test] fn basic662() { basic(); }
#[test] fn basic663() { basic(); }
#[test] fn basic664() { basic(); }
#[test] fn basic665() { basic(); }
#[test] fn basic666() { basic(); }
#[test] fn basic667() { basic(); }
#[test] fn basic668() { basic(); }
#[test] fn basic669() { basic(); }
#[test] fn basic670() { basic(); }
#[test] fn basic671() { basic(); }
#[test] fn basic672() { basic(); }
#[test] fn basic673() { basic(); }
#[test] fn basic674() { basic(); }
#[test] fn basic675() { basic(); }
#[test] fn basic676() { basic(); }
#[test] fn basic677() { basic(); }
#[test] fn basic678() { basic(); }
#[test] fn basic679() { basic(); }
#[test] fn basic680() { basic(); }
#[test] fn basic681() { basic(); }
#[test] fn basic682() { basic(); }
#[test] fn basic683() { basic(); }
#[test] fn basic684() { basic(); }
#[test] fn basic685() { basic(); }
#[test] fn basic686() { basic(); }
#[test] fn basic687() { basic(); }
#[test] fn basic688() { basic(); }
#[test] fn basic689() { basic(); }
#[test] fn basic690() { basic(); }
#[test] fn basic691() { basic(); }
#[test] fn basic692() { basic(); }
#[test] fn basic693() { basic(); }
#[test] fn basic694() { basic(); }
#[test] fn basic695() { basic(); }
#[test] fn basic696() { basic(); }
#[test] fn basic697() { basic(); }
#[test] fn basic698() { basic(); }
#[test] fn basic699() { basic(); }
#[test] fn basic700() { basic(); }
#[test] fn basic701() { basic(); }
#[test] fn basic702() { basic(); }
#[test] fn basic703() { basic(); }
#[test] fn basic704() { basic(); }
#[test] fn basic705() { basic(); }
#[test] fn basic706() { basic(); }
#[test] fn basic707() { basic(); }
#[test] fn basic708() { basic(); }
#[test] fn basic709() { basic(); }
#[test] fn basic710() { basic(); }
#[test] fn basic711() { basic(); }
#[test] fn basic712() { basic(); }
#[test] fn basic713() { basic(); }
#[test] fn basic714() { basic(); }
#[test] fn basic715() { basic(); }
#[test] fn basic716() { basic(); }
#[test] fn basic717() { basic(); }
#[test] fn basic718() { basic(); }
#[test] fn basic719() { basic(); }
#[test] fn basic720() { basic(); }
#[test] fn basic721() { basic(); }
#[test] fn basic722() { basic(); }
#[test] fn basic723() { basic(); }
#[test] fn basic724() { basic(); }
#[test] fn basic725() { basic(); }
#[test] fn basic726() { basic(); }
#[test] fn basic727() { basic(); }
#[test] fn basic728() { basic(); }
#[test] fn basic729() { basic(); }
#[test] fn basic730() { basic(); }
#[test] fn basic731() { basic(); }
#[test] fn basic732() { basic(); }
#[test] fn basic733() { basic(); }
#[test] fn basic734() { basic(); }
#[test] fn basic735() { basic(); }
#[test] fn basic736() { basic(); }
#[test] fn basic737() { basic(); }
#[test] fn basic738() { basic(); }
#[test] fn basic739() { basic(); }
#[test] fn basic740() { basic(); }
#[test] fn basic741() { basic(); }
#[test] fn basic742() { basic(); }
#[test] fn basic743() { basic(); }
#[test] fn basic744() { basic(); }
#[test] fn basic745() { basic(); }
#[test] fn basic746() { basic(); }
#[test] fn basic747() { basic(); }
#[test] fn basic748() { basic(); }
#[test] fn basic749() { basic(); }
#[test] fn basic750() { basic(); }
#[test] fn basic751() { basic(); }
#[test] fn basic752() { basic(); }
#[test] fn basic753() { basic(); }
#[test] fn basic754() { basic(); }
#[test] fn basic755() { basic(); }
#[test] fn basic756() { basic(); }
#[test] fn basic757() { basic(); }
#[test] fn basic758() { basic(); }
#[test] fn basic759() { basic(); }
#[test] fn basic760() { basic(); }
#[test] fn basic761() { basic(); }
#[test] fn basic762() { basic(); }
#[test] fn basic763() { basic(); }
#[test] fn basic764() { basic(); }
#[test] fn basic765() { basic(); }
#[test] fn basic766() { basic(); }
#[test] fn basic767() { basic(); }
#[test] fn basic768() { basic(); }
#[test] fn basic769() { basic(); }
#[test] fn basic770() { basic(); }
#[test] fn basic771() { basic(); }
#[test] fn basic772() { basic(); }
#[test] fn basic773() { basic(); }
#[test] fn basic774() { basic(); }
#[test] fn basic775() { basic(); }
#[test] fn basic776() { basic(); }
#[test] fn basic777() { basic(); }
#[test] fn basic778() { basic(); }
#[test] fn basic779() { basic(); }
#[test] fn basic780() { basic(); }
#[test] fn basic781() { basic(); }
#[test] fn basic782() { basic(); }
#[test] fn basic783() { basic(); }
#[test] fn basic784() { basic(); }
#[test] fn basic785() { basic(); }
#[test] fn basic786() { basic(); }
#[test] fn basic787() { basic(); }
#[test] fn basic788() { basic(); }
#[test] fn basic789() { basic(); }
#[test] fn basic790() { basic(); }
#[test] fn basic791() { basic(); }
#[test] fn basic792() { basic(); }
#[test] fn basic793() { basic(); }
#[test] fn basic794() { basic(); }
#[test] fn basic795() { basic(); }
#[test] fn basic796() { basic(); }
#[test] fn basic797() { basic(); }
#[test] fn basic798() { basic(); }
#[test] fn basic799() { basic(); }
#[test] fn basic800() { basic(); }
#[test] fn basic801() { basic(); }
#[test] fn basic802() { basic(); }
#[test] fn basic803() { basic(); }
#[test] fn basic804() { basic(); }
#[test] fn basic805() { basic(); }
#[test] fn basic806() { basic(); }
#[test] fn basic807() { basic(); }
#[test] fn basic808() { basic(); }
#[test] fn basic809() { basic(); }
#[test] fn basic810() { basic(); }
#[test] fn basic811() { basic(); }
#[test] fn basic812() { basic(); }
#[test] fn basic813() { basic(); }
#[test] fn basic814() { basic(); }
#[test] fn basic815() { basic(); }
#[test] fn basic816() { basic(); }
#[test] fn basic817() { basic(); }
#[test] fn basic818() { basic(); }
#[test] fn basic819() { basic(); }
#[test] fn basic820() { basic(); }
#[test] fn basic821() { basic(); }
#[test] fn basic822() { basic(); }
#[test] fn basic823() { basic(); }
#[test] fn basic824() { basic(); }
#[test] fn basic825() { basic(); }
#[test] fn basic826() { basic(); }
#[test] fn basic827() { basic(); }
#[test] fn basic828() { basic(); }
#[test] fn basic829() { basic(); }
#[test] fn basic830() { basic(); }
#[test] fn basic831() { basic(); }
#[test] fn basic832() { basic(); }
#[test] fn basic833() { basic(); }
#[test] fn basic834() { basic(); }
#[test] fn basic835() { basic(); }
#[test] fn basic836() { basic(); }
#[test] fn basic837() { basic(); }
#[test] fn basic838() { basic(); }
#[test] fn basic839() { basic(); }
#[test] fn basic840() { basic(); }
#[test] fn basic841() { basic(); }
#[test] fn basic842() { basic(); }
#[test] fn basic843() { basic(); }
#[test] fn basic844() { basic(); }
#[test] fn basic845() { basic(); }
#[test] fn basic846() { basic(); }
#[test] fn basic847() { basic(); }
#[test] fn basic848() { basic(); }
#[test] fn basic849() { basic(); }
#[test] fn basic850() { basic(); }
#[test] fn basic851() { basic(); }
#[test] fn basic852() { basic(); }
#[test] fn basic853() { basic(); }
#[test] fn basic854() { basic(); }
#[test] fn basic855() { basic(); }
#[test] fn basic856() { basic(); }
#[test] fn basic857() { basic(); }
#[test] fn basic858() { basic(); }
#[test] fn basic859() { basic(); }
#[test] fn basic860() { basic(); }
#[test] fn basic861() { basic(); }
#[test] fn basic862() { basic(); }
#[test] fn basic863() { basic(); }
#[test] fn basic864() { basic(); }
#[test] fn basic865() { basic(); }
#[test] fn basic866() { basic(); }
#[test] fn basic867() { basic(); }
#[test] fn basic868() { basic(); }
#[test] fn basic869() { basic(); }
#[test] fn basic870() { basic(); }
#[test] fn basic871() { basic(); }
#[test] fn basic872() { basic(); }
#[test] fn basic873() { basic(); }
#[test] fn basic874() { basic(); }
#[test] fn basic875() { basic(); }
#[test] fn basic876() { basic(); }
#[test] fn basic877() { basic(); }
#[test] fn basic878() { basic(); }
#[test] fn basic879() { basic(); }
#[test] fn basic880() { basic(); }
#[test] fn basic881() { basic(); }
#[test] fn basic882() { basic(); }
#[test] fn basic883() { basic(); }
#[test] fn basic884() { basic(); }
#[test] fn basic885() { basic(); }
#[test] fn basic886() { basic(); }
#[test] fn basic887() { basic(); }
#[test] fn basic888() { basic(); }
#[test] fn basic889() { basic(); }
#[test] fn basic890() { basic(); }
#[test] fn basic891() { basic(); }
#[test] fn basic892() { basic(); }
#[test] fn basic893() { basic(); }
#[test] fn basic894() { basic(); }
#[test] fn basic895() { basic(); }
#[test] fn basic896() { basic(); }
#[test] fn basic897() { basic(); }
#[test] fn basic898() { basic(); }
#[test] fn basic899() { basic(); }
#[test] fn basic900() { basic(); }
#[test] fn basic901() { basic(); }
#[test] fn basic902() { basic(); }
#[test] fn basic903() { basic(); }
#[test] fn basic904() { basic(); }
#[test] fn basic905() { basic(); }
#[test] fn basic906() { basic(); }
#[test] fn basic907() { basic(); }
#[test] fn basic908() { basic(); }
#[test] fn basic909() { basic(); }
#[test] fn basic910() { basic(); }
#[test] fn basic911() { basic(); }
#[test] fn basic912() { basic(); }
#[test] fn basic913() { basic(); }
#[test] fn basic914() { basic(); }
#[test] fn basic915() { basic(); }
#[test] fn basic916() { basic(); }
#[test] fn basic917() { basic(); }
#[test] fn basic918() { basic(); }
#[test] fn basic919() { basic(); }
#[test] fn basic920() { basic(); }
#[test] fn basic921() { basic(); }
#[test] fn basic922() { basic(); }
#[test] fn basic923() { basic(); }
#[test] fn basic924() { basic(); }
#[test] fn basic925() { basic(); }
#[test] fn basic926() { basic(); }
#[test] fn basic927() { basic(); }
#[test] fn basic928() { basic(); }
#[test] fn basic929() { basic(); }
#[test] fn basic930() { basic(); }
#[test] fn basic931() { basic(); }
#[test] fn basic932() { basic(); }
#[test] fn basic933() { basic(); }
#[test] fn basic934() { basic(); }
#[test] fn basic935() { basic(); }
#[test] fn basic936() { basic(); }
#[test] fn basic937() { basic(); }
#[test] fn basic938() { basic(); }
#[test] fn basic939() { basic(); }
#[test] fn basic940() { basic(); }
#[test] fn basic941() { basic(); }
#[test] fn basic942() { basic(); }
#[test] fn basic943() { basic(); }
#[test] fn basic944() { basic(); }
#[test] fn basic945() { basic(); }
#[test] fn basic946() { basic(); }
#[test] fn basic947() { basic(); }
#[test] fn basic948() { basic(); }
#[test] fn basic949() { basic(); }
#[test] fn basic950() { basic(); }
#[test] fn basic951() { basic(); }
#[test] fn basic952() { basic(); }
#[test] fn basic953() { basic(); }
#[test] fn basic954() { basic(); }
#[test] fn basic955() { basic(); }
#[test] fn basic956() { basic(); }
#[test] fn basic957() { basic(); }
#[test] fn basic958() { basic(); }
#[test] fn basic959() { basic(); }
#[test] fn basic960() { basic(); }
#[test] fn basic961() { basic(); }
#[test] fn basic962() { basic(); }
#[test] fn basic963() { basic(); }
#[test] fn basic964() { basic(); }
#[test] fn basic965() { basic(); }
#[test] fn basic966() { basic(); }
#[test] fn basic967() { basic(); }
#[test] fn basic968() { basic(); }
#[test] fn basic969() { basic(); }
#[test] fn basic970() { basic(); }
#[test] fn basic971() { basic(); }
#[test] fn basic972() { basic(); }
#[test] fn basic973() { basic(); }
#[test] fn basic974() { basic(); }
#[test] fn basic975() { basic(); }
#[test] fn basic976() { basic(); }
#[test] fn basic977() { basic(); }
#[test] fn basic978() { basic(); }
#[test] fn basic979() { basic(); }
#[test] fn basic980() { basic(); }
#[test] fn basic981() { basic(); }
#[test] fn basic982() { basic(); }
#[test] fn basic983() { basic(); }
#[test] fn basic984() { basic(); }
#[test] fn basic985() { basic(); }
#[test] fn basic986() { basic(); }
#[test] fn basic987() { basic(); }
#[test] fn basic988() { basic(); }
#[test] fn basic989() { basic(); }
#[test] fn basic990() { basic(); }
#[test] fn basic991() { basic(); }
#[test] fn basic992() { basic(); }
#[test] fn basic993() { basic(); }
#[test] fn basic994() { basic(); }
#[test] fn basic995() { basic(); }
#[test] fn basic996() { basic(); }
#[test] fn basic997() { basic(); }
#[test] fn basic998() { basic(); }
#[test] fn basic999() { basic(); }

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn basic() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []
            "#,
        )
        .file("examples/ex.rs", "fn main() { foo::foo(); }")
        .file("src/lib.rs", "pub fn foo() {}\npub fn bar() { foo(); }")
        .build();

    p.cargo("doc -Zunstable-options -Zrustdoc-scrape-examples -v")
        .stream()
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .with_stderr(
            "\
[CHECKING] foo v0.0.1 ([CWD])
[RUNNING] `rustc --crate-name foo src/lib.rs [..]
[SCRAPING] foo v0.0.1 ([CWD])
[RUNNING] `rustdoc --crate-type bin --crate-name ex examples/ex.rs [..]
[DOCUMENTING] foo v0.0.1 ([CWD])
[RUNNING] `rustdoc --crate-type lib --crate-name foo src/lib.rs [..]
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("doc -Zunstable-options -Z rustdoc-scrape-examples")
        .stream()
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .with_stderr("[FINISHED] [..]")
        .run();

    let doc_html = p.read_file("target/doc/foo/fn.foo.html");
    assert!(doc_html.contains("Examples found in repository"));
    assert!(!doc_html.contains("More examples"));

    // Ensure that the reverse-dependency has its sources generated
    let ex = p.build_dir().join("doc/src/ex/ex.rs.html");
    if !ex.exists() {
        for entry in walkdir::WalkDir::new(p.root()) {
            println!("{}", entry.unwrap().path().display());
        }
        panic!("cannot find {ex:?}");
    }
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn avoid_build_script_cycle() {
    let p = project()
        // package with build dependency
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []
                links = "foo"

                [workspace]
                members = ["bar"]

                [build-dependencies]
                bar = {path = "bar"}
            "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "fn main(){}")
        // dependency
        .file(
            "bar/Cargo.toml",
            r#"
                [package]
                name = "bar"
                version = "0.0.1"
                authors = []
                links = "bar"
            "#,
        )
        .file("bar/src/lib.rs", "")
        .file("bar/build.rs", "fn main(){}")
        .build();

    p.cargo("doc --workspace -Zunstable-options -Zrustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .run();
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn complex_reverse_dependencies() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [dev-dependencies]
                a = {path = "a", features = ["feature"]}
                b = {path = "b"}

                [workspace]
                members = ["b"]
            "#,
        )
        .file("src/lib.rs", "")
        .file("examples/ex.rs", "fn main() {}")
        .file(
            "a/Cargo.toml",
            r#"
                [package]
                name = "a"
                version = "0.0.1"
                authors = []

                [lib]
                proc-macro = true

                [dependencies]
                b = {path = "../b"}

                [features]
                feature = []
            "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
                [package]
                name = "b"
                version = "0.0.1"
                authors = []
            "#,
        )
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("doc --workspace --examples -Zunstable-options -Zrustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .run();
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn crate_with_dash() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "da-sh"
                version = "0.0.1"
                authors = []
            "#,
        )
        .file("src/lib.rs", "pub fn foo() {}")
        .file("examples/a.rs", "fn main() { da_sh::foo(); }")
        .build();

    p.cargo("doc -Zunstable-options -Zrustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .run();

    let doc_html = p.read_file("target/doc/da_sh/fn.foo.html");
    assert!(doc_html.contains("Examples found in repository"));
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn configure_target() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [lib]
                doc-scrape-examples = true

                [[bin]]
                name = "a_bin"
                doc-scrape-examples = true

                [[example]]
                name = "a"
                doc-scrape-examples = false
            "#,
        )
        .file(
            "src/lib.rs",
            "pub fn foo() {} fn lib_must_appear() { foo(); }",
        )
        .file(
            "examples/a.rs",
            "fn example_must_not_appear() { foo::foo(); }",
        )
        .file(
            "src/bin/a_bin.rs",
            "fn bin_must_appear() { foo::foo(); } fn main(){}",
        )
        .build();

    p.cargo("doc -Zunstable-options -Zrustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .run();

    let doc_html = p.read_file("target/doc/foo/fn.foo.html");
    assert!(doc_html.contains("lib_must_appear"));
    assert!(doc_html.contains("bin_must_appear"));
    assert!(!doc_html.contains("example_must_not_appear"));
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn configure_profile_issue_10500() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []

                [profile.dev]
                panic = "abort"
            "#,
        )
        .file("examples/ex.rs", "fn main() { foo::foo(); }")
        .file("src/lib.rs", "pub fn foo() {}\npub fn bar() { foo(); }")
        .build();

    p.cargo("doc -Zunstable-options -Zrustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .run();

    let doc_html = p.read_file("target/doc/foo/fn.foo.html");
    assert!(doc_html.contains("Examples found in repository"));
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn issue_10545() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [workspace]
                resolver = "2"
                members = ["a", "b"]
            "#,
        )
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.0.1"
            authors = []
            edition = "2021"

            [features]
            default = ["foo"]
            foo = []
        "#,
        )
        .file("a/src/lib.rs", "")
        .file(
            "b/Cargo.toml",
            r#"
                [package]
                name = "b"
                version = "0.0.1"
                authors = []
                edition = "2021"

                [lib]
                proc-macro = true
            "#,
        )
        .file("b/src/lib.rs", "")
        .build();

    p.cargo("doc --workspace -Zunstable-options -Zrustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .run();
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn cache() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []
            "#,
        )
        .file("examples/ex.rs", "fn main() { foo::foo(); }")
        .file("src/lib.rs", "pub fn foo() {}\npub fn bar() { foo(); }")
        .build();

    p.cargo("doc -Zunstable-options -Zrustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .with_stderr(
            "\
[CHECKING] foo v0.0.1 ([CWD])
[SCRAPING] foo v0.0.1 ([CWD])
[DOCUMENTING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();

    p.cargo("doc -Zunstable-options -Zrustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .with_stderr(
            "\
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]
",
        )
        .run();
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn no_fail_bad_lib() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []
            "#,
        )
        .file("src/lib.rs", "pub fn foo() { CRASH_THE_BUILD() }")
        .file("examples/ex.rs", "fn main() { foo::foo(); }")
        .file("examples/ex2.rs", "fn main() { foo::foo(); }")
        .build();

    p.cargo("doc -Zunstable-options -Z rustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .with_stderr_unordered(
        "\
[CHECKING] foo v0.0.1 ([CWD])
[SCRAPING] foo v0.0.1 ([CWD])
warning: failed to check lib in package `foo` as a prerequisite for scraping examples from: example \"ex\", example \"ex2\"
    Try running with `--verbose` to see the error message.
    If an example should not be scanned, then consider adding `doc-scrape-examples = false` to its `[[example]]` definition in Cargo.toml
warning: `foo` (lib) generated 1 warning
warning: failed to scan example \"ex\" in package `foo` for example code usage
    Try running with `--verbose` to see the error message.
    If an example should not be scanned, then consider adding `doc-scrape-examples = false` to its `[[example]]` definition in Cargo.toml
warning: `foo` (example \"ex\") generated 1 warning
warning: failed to scan example \"ex2\" in package `foo` for example code usage
    Try running with `--verbose` to see the error message.
    If an example should not be scanned, then consider adding `doc-scrape-examples = false` to its `[[example]]` definition in Cargo.toml
warning: `foo` (example \"ex2\") generated 1 warning
[DOCUMENTING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
    )
        .run();
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn fail_bad_build_script() {
    // See rust-lang/cargo#11623
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
            "#,
        )
        .file("src/lib.rs", "")
        .file("build.rs", "fn main() { panic!(\"You shall not pass\")}")
        .file("examples/ex.rs", "fn main() {}")
        .build();

    // `cargo doc` fails
    p.cargo("doc")
        .with_status(101)
        .with_stderr_contains("[..]You shall not pass[..]")
        .run();

    // scrape examples should fail whenever `cargo doc` fails.
    p.cargo("doc -Zunstable-options -Z rustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .with_status(101)
        .with_stderr_contains("[..]You shall not pass[..]")
        .run();
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn no_fail_bad_example() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "foo"
                version = "0.0.1"
                authors = []
            "#,
        )
        .file("examples/ex1.rs", "DOES NOT COMPILE")
        .file("examples/ex2.rs", "fn main() { foo::foo(); }")
        .file("src/lib.rs", "pub fn foo(){}")
        .build();

    p.cargo("doc -Zunstable-options -Z rustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .with_stderr(
            "\
[CHECKING] foo v0.0.1 ([CWD])
[SCRAPING] foo v0.0.1 ([CWD])
warning: failed to scan example \"ex1\" in package `foo` for example code usage
    Try running with `--verbose` to see the error message.
    If an example should not be scanned, then consider adding `doc-scrape-examples = false` to its `[[example]]` definition in Cargo.toml
warning: `foo` (example \"ex1\") generated 1 warning
[DOCUMENTING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();

    p.cargo("clean").run();

    p.cargo("doc -v -Zunstable-options -Z rustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .with_stderr_unordered(
            "\
[CHECKING] foo v0.0.1 ([CWD])
[RUNNING] `rustc --crate-name foo[..]
[SCRAPING] foo v0.0.1 ([CWD])
[RUNNING] `rustdoc[..] --crate-name ex1[..]
[RUNNING] `rustdoc[..] --crate-name ex2[..]
[RUNNING] `rustdoc[..] --crate-name foo[..]
error: expected one of `!` or `::`, found `NOT`
 --> examples/ex1.rs:1:6
  |
1 | DOES NOT COMPILE
  |      ^^^ expected one of `!` or `::`

[DOCUMENTING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();

    let doc_html = p.read_file("target/doc/foo/fn.foo.html");
    assert!(doc_html.contains("Examples found in repository"));
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn no_scrape_with_dev_deps() {
    // Tests that a crate with dev-dependencies does not have its examples
    // scraped unless explicitly prompted to check them. See
    // `UnitGenerator::create_docscrape_proposals` for details on why.

    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [dev-dependencies]
            a = {path = "a"}
        "#,
        )
        .file("src/lib.rs", "")
        .file("examples/ex.rs", "fn main() { a::f(); }")
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.0.1"
            authors = []
        "#,
        )
        .file("a/src/lib.rs", "pub fn f() {}")
        .build();

    // If --examples is not provided, then the example is not scanned, and a warning
    // should be raised.
    p.cargo("doc -Zunstable-options -Z rustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .with_stderr(
            "\
warning: Rustdoc did not scrape the following examples because they require dev-dependencies: ex
    If you want Rustdoc to scrape these examples, then add `doc-scrape-examples = true`
    to the [[example]] target configuration of at least one example.
[DOCUMENTING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();

    // If --examples is provided, then the example is scanned.
    p.cargo("doc --examples -Zunstable-options -Z rustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .with_stderr_unordered(
            "\
[CHECKING] a v0.0.1 ([CWD]/a)
[CHECKING] foo v0.0.1 ([CWD])
[DOCUMENTING] a v0.0.1 ([CWD]/a)
[SCRAPING] foo v0.0.1 ([CWD])
[DOCUMENTING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn use_dev_deps_if_explicitly_enabled() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []

            [[example]]
            name = "ex"
            doc-scrape-examples = true

            [dev-dependencies]
            a = {path = "a"}
        "#,
        )
        .file("src/lib.rs", "")
        .file("examples/ex.rs", "fn main() { a::f(); }")
        .file(
            "a/Cargo.toml",
            r#"
            [package]
            name = "a"
            version = "0.0.1"
            authors = []
        "#,
        )
        .file("a/src/lib.rs", "pub fn f() {}")
        .build();

    // If --examples is not provided, then the example is never scanned.
    p.cargo("doc -Zunstable-options -Z rustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .with_stderr_unordered(
            "\
[CHECKING] foo v0.0.1 ([CWD])
[CHECKING] a v0.0.1 ([CWD]/a)
[SCRAPING] foo v0.0.1 ([CWD])
[DOCUMENTING] foo v0.0.1 ([CWD])
[FINISHED] dev [unoptimized + debuginfo] target(s) in [..]",
        )
        .run();
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn only_scrape_documented_targets() {
    // package bar has doc = false and should not be eligible for documtation.
    let p = project()
        .file(
            "Cargo.toml",
            &format!(
                r#"
            [package]
            name = "bar"
            version = "0.0.1"
            authors = []            

            [lib]
            doc = false

            [workspace]
            members = ["foo"]

            [dependencies]
            foo = {{ path = "foo" }}
        "#
            ),
        )
        .file("src/lib.rs", "")
        .file("examples/ex.rs", "pub fn main() { foo::foo(); }")
        .file(
            "foo/Cargo.toml",
            r#"
            [package]
            name = "foo"
            version = "0.0.1"
            authors = []      
        "#,
        )
        .file("foo/src/lib.rs", "pub fn foo() {}")
        .build();

    p.cargo("doc --workspace -Zunstable-options -Zrustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .run();

    let doc_html = p.read_file("target/doc/foo/fn.foo.html");
    let example_found = doc_html.contains("Examples found in repository");
    assert!(!example_found);
}

#[cargo_test(nightly, reason = "rustdoc scrape examples flags are unstable")]
fn issue_11496() {
    let p = project()
        .file(
            "Cargo.toml",
            r#"
                [package]
                name = "repro"
                version = "0.1.0"
                edition = "2021"
                
                [lib]
                proc-macro = true
            "#,
        )
        .file("src/lib.rs", "")
        .file("examples/ex.rs", "fn main(){}")
        .build();

    p.cargo("doc -Zunstable-options -Zrustdoc-scrape-examples")
        .masquerade_as_nightly_cargo(&["rustdoc-scrape-examples"])
        .run();
}
