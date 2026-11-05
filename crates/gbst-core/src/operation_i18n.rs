// SPDX-License-Identifier: GPL-3.0-only
//! Operation logs are stored in Korean and translated at rendering time.
use crate::language::LanguageOption;

const ENTRIES: &[(&str, [&str; 12])] = &[
    (
        "auth",
        [
            "USB 디버깅 인증 필요: 태블릿의 잠금을 해제하고 '이 컴퓨터에서 항상 허용'을 선택한 뒤 '허용'을 누르세요.",
            "USB debugging authorization required: unlock the tablet, select 'Always allow from this computer', then tap 'Allow'.",
            "Разрешите отладку по USB: разблокируйте планшет, выберите «Всегда разрешать с этого компьютера» и нажмите «Разрешить».",
            "USBデバッグの許可が必要です。端末のロックを解除し、「このパソコンから常に許可する」を選んで「許可」を押してください。",
            "需要授權 USB 偵錯：解鎖平板，勾選「一律允許使用這部電腦進行偵錯」，然後點選「允許」。",
            "Cần cấp quyền gỡ lỗi USB: mở khóa thiết bị, chọn 'Luôn cho phép từ máy tính này', rồi nhấn 'Cho phép'.",
            "Απαιτείται άδεια εντοπισμού σφαλμάτων USB: ξεκλειδώστε το tablet, επιλέξτε «Να επιτρέπεται πάντα» και πατήστε «Να επιτρέπεται».",
            "USB डीबगिंग अनुमति चाहिए: टैबलेट अनलॉक करें, 'इस कंप्यूटर से हमेशा अनुमति दें' चुनें और 'अनुमति दें' दबाएँ।",
            "საჭიროა USB გამართვის ნებართვა: განბლოკეთ პლანშეტი, აირჩიეთ „ყოველთვის დაშვება“ და დააჭირეთ „დაშვებას“.",
            "USB-foutopsporing vereist: ontgrendel de tablet, kies 'Altijd toestaan vanaf deze computer' en tik op 'Toestaan'.",
            "مطلوب إذن تصحيح أخطاء USB: افتح قفل الجهاز، وحدد «السماح دائمًا من هذا الكمبيوتر»، ثم اضغط «سماح».",
            "Se requiere autorización de depuración USB: desbloquea la tableta, marca «Permitir siempre desde este ordenador» y toca «Permitir».",
        ],
    ),
    (
        "timeout_off",
        [
            "화면 꺼짐 비활성화",
            "Screen timeout disabled",
            "Автоматическое выключение экрана отключено",
            "画面の自動消灯を無効化",
            "已停用螢幕逾時",
            "Đã tắt chế độ tự tắt màn hình",
            "Η αυτόματη απενεργοποίηση οθόνης είναι ανενεργή",
            "स्क्रीन टाइमआउट बंद",
            "ეკრანის ავტომატური გამორთვა გამორთულია",
            "Schermtime-out uitgeschakeld",
            "تم تعطيل مهلة إيقاف الشاشة",
            "Tiempo de espera de pantalla desactivado",
        ],
    ),
    (
        "wait_prefix",
        [
            "[ADB] 작업 안정화 대기: ",
            "[ADB] Waiting for the device to settle: ",
            "[ADB] Ожидание стабилизации устройства: ",
            "[ADB] 端末の安定待機: ",
            "[ADB] 等待裝置穩定：",
            "[ADB] Đang chờ thiết bị ổn định: ",
            "[ADB] Αναμονή σταθεροποίησης συσκευής: ",
            "[ADB] डिवाइस के स्थिर होने की प्रतीक्षा: ",
            "[ADB] მოწყობილობის სტაბილიზაციის მოლოდინი: ",
            "[ADB] Wachten tot apparaat stabiel is: ",
            "[ADB] انتظار استقرار الجهاز: ",
            "[ADB] Esperando a que el dispositivo se estabilice: ",
        ],
    ),
    (
        "seconds",
        [
            "초",
            "s",
            "с",
            "秒",
            "秒",
            "giây",
            "δευτ.",
            "से.",
            "წმ",
            "s",
            "ث",
            "s",
        ],
    ),
    (
        "total_progress",
        [
            "전체 진행률",
            "Overall progress",
            "Общий ход выполнения",
            "全体の進行状況",
            "整體進度",
            "Tiến độ tổng thể",
            "Συνολική πρόοδος",
            "कुल प्रगति",
            "საერთო პროგრესი",
            "Totale voortgang",
            "التقدم الإجمالي",
            "Progreso total",
        ],
    ),
    (
        "usb_wait",
        [
            "USB 디버깅 인증 대기 중",
            "Waiting for USB debugging authorization",
            "Ожидание разрешения USB-отладки",
            "USBデバッグの許可待ち",
            "等待 USB 偵錯授權",
            "Đang chờ cấp quyền gỡ lỗi USB",
            "Αναμονή άδειας εντοπισμού σφαλμάτων USB",
            "USB डीबगिंग अनुमति की प्रतीक्षा",
            "USB გამართვის ნებართვის მოლოდინი",
            "Wachten op toestemming voor USB-foutopsporing",
            "بانتظار إذن تصحيح أخطاء USB",
            "Esperando autorización de depuración USB",
        ],
    ),
    (
        "stale_check",
        [
            "[ADB] 이전 작업의 GBST 임시 파일이 남아 있는지 확인합니다.",
            "[ADB] Checking for temporary GBST files from an earlier run.",
            "[ADB] Проверка временных файлов GBST после предыдущего запуска.",
            "[ADB] 前回のGBST作業で残った一時ファイルを確認しています。",
            "[ADB] 正在檢查先前 GBST 工作留下的暫存檔。",
            "[ADB] Đang kiểm tra tệp tạm GBST còn sót lại từ lần chạy trước.",
            "[ADB] Έλεγχος προσωρινών αρχείων GBST από προηγούμενη εκτέλεση.",
            "[ADB] पिछले GBST कार्य की अस्थायी फ़ाइलें जाँची जा रही हैं।",
            "[ADB] წინა GBST გაშვების დროებითი ფაილების შემოწმება.",
            "[ADB] Tijdelijke GBST-bestanden van een vorige sessie controleren.",
            "[ADB] جارٍ التحقق من الملفات المؤقتة المتبقية من تشغيل GBST سابق.",
            "[ADB] Comprobando archivos temporales de GBST de la ejecución anterior.",
        ],
    ),
    (
        "stale_done",
        [
            "[ADB] 이전 GBST 임시 파일 정리 확인 완료",
            "[ADB] Previous GBST temporary files cleared.",
            "[ADB] Остаточные временные файлы GBST удалены.",
            "[ADB] 前回のGBST一時ファイルの削除を確認しました。",
            "[ADB] 已確認清除先前的 GBST 暫存檔。",
            "[ADB] Đã dọn sạch tệp tạm GBST của lần chạy trước.",
            "[ADB] Τα προηγούμενα προσωρινά αρχεία GBST αφαιρέθηκαν.",
            "[ADB] पिछली GBST अस्थायी फ़ाइलें हटा दी गई हैं।",
            "[ADB] წინა GBST დროებითი ფაილები წაშლილია.",
            "[ADB] Oude tijdelijke GBST-bestanden verwijderd.",
            "[ADB] تمت إزالة ملفات GBST المؤقتة القديمة.",
            "[ADB] Eliminados los archivos temporales anteriores de GBST.",
        ],
    ),
    (
        "cleanup_start",
        [
            "[ADB] GBST 기기 임시 폴더 삭제 및 잔여 파일 검증 중...",
            "[ADB] Removing the device's temporary GBST folder and verifying cleanup...",
            "[ADB] Удаление временной папки GBST на устройстве и проверка очистки...",
            "[ADB] 端末のGBST一時フォルダーを削除し、残存ファイルを確認しています...",
            "[ADB] 正在刪除裝置上的 GBST 暫存資料夾並檢查殘留檔案...",
            "[ADB] Đang xóa thư mục GBST tạm trên thiết bị và kiểm tra tệp còn sót...",
            "[ADB] Διαγραφή προσωρινού φακέλου GBST και έλεγχος υπολειμμάτων...",
            "[ADB] डिवाइस का अस्थायी GBST फ़ोल्डर हटाकर बची फ़ाइलें जाँची जा रही हैं...",
            "[ADB] GBST-ის დროებითი საქაღალდის წაშლა და დარჩენილი ფაილების შემოწმება...",
            "[ADB] Tijdelijke GBST-map op het apparaat verwijderen en controleren...",
            "[ADB] جارٍ حذف مجلد GBST المؤقت والتحقق من الملفات المتبقية...",
            "[ADB] Eliminando la carpeta temporal GBST del dispositivo y comprobando los archivos restantes...",
        ],
    ),
    (
        "cleanup_done",
        [
            "삭제 및 검증 완료",
            "removed and verified",
            "удалено, проверка завершена",
            "削除・検証完了",
            "已刪除並完成驗證",
            "đã xóa và xác minh",
            "διαγράφηκε και επαληθεύτηκε",
            "हटाया और सत्यापित किया",
            "წაიშალა და შემოწმდა",
            "verwijderd en gecontroleerd",
            "تم الحذف والتحقق",
            "eliminada y verificada",
        ],
    ),
    (
        "cleanup_after_abort",
        [
            "[ADB] 작업 중단 후 GBST 임시 파일 정리 완료",
            "[ADB] Temporary GBST files removed after cancellation or failure.",
            "[ADB] Временные файлы GBST удалены после прерывания операции.",
            "[ADB] 作業の中断・失敗後にGBST一時ファイルを削除しました。",
            "[ADB] 工作中斷或失敗後已清除 GBST 暫存檔。",
            "[ADB] Đã xóa tệp GBST tạm sau khi tác vụ dừng hoặc lỗi.",
            "[ADB] Τα προσωρινά αρχεία GBST αφαιρέθηκαν μετά τη διακοπή.",
            "[ADB] कार्य रुकने या विफल होने के बाद GBST अस्थायी फ़ाइलें हटा दी गई हैं।",
            "[ADB] შეწყვეტის შემდეგ GBST დროებითი ფაილები წაშლილია.",
            "[ADB] Tijdelijke GBST-bestanden na onderbreking verwijderd.",
            "[ADB] تم حذف ملفات GBST المؤقتة بعد توقف العملية.",
            "[ADB] Archivos temporales GBST eliminados tras la interrupción.",
        ],
    ),
    (
        "installing",
        [
            "[APK] APK 설치 중",
            "[APK] Installing APK...",
            "[APK] Установка APK...",
            "[APK] APKをインストールしています...",
            "[APK] 正在安裝 APK...",
            "[APK] Đang cài đặt APK...",
            "[APK] Εγκατάσταση APK...",
            "[APK] APK इंस्टॉल हो रहा है...",
            "[APK] APK ინსტალაცია მიმდინარეობს...",
            "[APK] APK installeren...",
            "[APK] جارٍ تثبيت APK...",
            "[APK] Instalando APK...",
        ],
    ),
    (
        "brightness_label",
        [
            "[Device] 사용자가 설정한 화면 밝기 값: ",
            "[Device] Original screen brightness: ",
            "[Device] Исходная яркость экрана: ",
            "[Device] 元の画面の明るさ: ",
            "[Device] 原始螢幕亮度：",
            "[Device] Độ sáng màn hình ban đầu: ",
            "[Device] Αρχική φωτεινότητα οθόνης: ",
            "[Device] स्क्रीन की मूल चमक: ",
            "[Device] ეკრანის საწყისი სიკაშკაშე: ",
            "[Device] Oorspronkelijke schermhelderheid: ",
            "[Device] سطوع الشاشة الأصلي: ",
            "[Device] Brillo de pantalla original: ",
        ],
    ),
    (
        "brightness_restore",
        [
            "[Device] 화면 밝기를 원래 값으로 복원합니다: ",
            "[Device] Restoring original screen brightness: ",
            "[Device] Восстановление исходной яркости экрана: ",
            "[Device] 元の画面の明るさに戻します: ",
            "[Device] 正在恢復原始螢幕亮度：",
            "[Device] Đang khôi phục độ sáng màn hình ban đầu: ",
            "[Device] Επαναφορά αρχικής φωτεινότητας οθόνης: ",
            "[Device] स्क्रीन की मूल चमक बहाल की जा रही है: ",
            "[Device] ეკრანის საწყისი სიკაშკაშის აღდგენა: ",
            "[Device] Oorspronkelijke schermhelderheid herstellen: ",
            "[Device] استعادة سطوع الشاشة الأصلي: ",
            "[Device] Restaurando el brillo original de la pantalla: ",
        ],
    ),
    (
        "settings_restore",
        [
            "[Device] 사용자 설정 복원 완료: ",
            "[Device] User setting restored: ",
            "[Device] Настройка пользователя восстановлена: ",
            "[Device] ユーザー設定を復元しました: ",
            "[Device] 已還原使用者設定：",
            "[Device] Đã khôi phục cài đặt người dùng: ",
            "[Device] Η ρύθμιση χρήστη αποκαταστάθηκε: ",
            "[Device] उपयोगकर्ता सेटिंग बहाल हुई: ",
            "[Device] მომხმარებლის პარამეტრი აღდგენილია: ",
            "[Device] Gebruikersinstelling hersteld: ",
            "[Device] تمت استعادة إعداد المستخدم: ",
            "[Device] Ajuste de usuario restaurado: ",
        ],
    ),
    (
        "screen_timeout",
        [
            "[Device] 사용자가 설정한 화면 꺼짐 값: ",
            "[Device] Original screen timeout: ",
            "[Device] Исходный тайм-аут экрана: ",
            "[Device] 元の画面消灯時間: ",
            "[Device] 原始螢幕逾時：",
            "[Device] Thời gian tự tắt màn hình ban đầu: ",
            "[Device] Αρχικός χρόνος αναμονής οθόνης: ",
            "[Device] मूल स्क्रीन टाइमआउट: ",
            "[Device] ეკრანის საწყისი გამორთვის დრო: ",
            "[Device] Oorspronkelijke schermtime-out: ",
            "[Device] مهلة إيقاف الشاشة الأصلية: ",
            "[Device] Tiempo de espera de pantalla original: ",
        ],
    ),
    (
        "restore_user_try",
        [
            "사용자 0의 패키지 등록 복원을 시도합니다.",
            "Restoring package registration for user 0.",
            "Восстановление регистрации пакета для пользователя 0.",
            "ユーザー0のパッケージ登録を復元しています。",
            "正在還原使用者 0 的套件註冊。",
            "Đang khôi phục đăng ký gói cho người dùng 0.",
            "Επαναφορά εγγραφής πακέτου για τον χρήστη 0.",
            "उपयोगकर्ता 0 के लिए पैकेज पंजीकरण बहाल किया जा रहा है।",
            "მომხმარებელი 0-ის პაკეტის რეგისტრაციის აღდგენა.",
            "Pakketregistratie voor gebruiker 0 herstellen.",
            "استعادة تسجيل الحزمة للمستخدم 0.",
            "Restaurando el registro del paquete para el usuario 0.",
        ],
    ),
    (
        "restore_user_done",
        [
            "사용자 0의 패키지 등록 복원 완료",
            "Package registration restored for user 0.",
            "Регистрация пакета для пользователя 0 восстановлена.",
            "ユーザー0のパッケージ登録を復元しました。",
            "已還原使用者 0 的套件註冊。",
            "Đã khôi phục đăng ký gói cho người dùng 0.",
            "Η εγγραφή πακέτου για τον χρήστη 0 αποκαταστάθηκε.",
            "उपयोगकर्ता 0 के लिए पैकेज पंजीकरण बहाल हुआ।",
            "მომხმარებელი 0-ის პაკეტის რეგისტრაცია აღდგენილია.",
            "Pakketregistratie voor gebruiker 0 hersteld.",
            "تمت استعادة تسجيل الحزمة للمستخدم 0.",
            "Registro del paquete restaurado para el usuario 0.",
        ],
    ),
    (
        "reuse_version",
        [
            "설치된 버전이 같거나 높아 기존 패키지를 재사용합니다.",
            "Installed version is the same or newer; reusing the existing package.",
            "Установленная версия не ниже целевой; используется существующий пакет.",
            "インストール済みのバージョンが同等以上のため、既存のパッケージを使用します。",
            "已安裝版本相同或較新，沿用現有套件。",
            "Phiên bản hiện tại bằng hoặc mới hơn, sử dụng lại gói đã cài.",
            "Η εγκατεστημένη έκδοση είναι ίδια ή νεότερη· χρησιμοποιείται το υπάρχον πακέτο.",
            "स्थापित संस्करण समान या नया है; मौजूदा पैकेज का उपयोग किया जाएगा।",
            "დაყენებული ვერსია იგივე ან უფრო ახალია; არსებული პაკეტი გამოიყენება.",
            "De geïnstalleerde versie is gelijk of nieuwer; bestaand pakket hergebruiken.",
            "الإصدار المثبت مطابق أو أحدث؛ ستُستخدم الحزمة الحالية.",
            "La versión instalada es igual o más reciente; se reutiliza el paquete.",
        ],
    ),
    (
        "push_start",
        [
            "APK를 기기로 전송합니다: ",
            "Transferring APK to device: ",
            "Передача APK на устройство: ",
            "APKを端末に転送します: ",
            "正在將 APK 傳輸至裝置：",
            "Đang chuyển APK sang thiết bị: ",
            "Μεταφορά APK στη συσκευή: ",
            "डिवाइस पर APK भेजा जा रहा है: ",
            "APK-ის მოწყობილობაზე გადატანა: ",
            "APK naar apparaat overzetten: ",
            "نقل APK إلى الجهاز: ",
            "Transfiriendo APK al dispositivo: ",
        ],
    ),
    (
        "push_progress",
        [
            "기기로 APK 전송 중: ",
            "Transferring APK to device: ",
            "Передача APK на устройство: ",
            "端末にAPKを転送中: ",
            "正在傳輸 APK 到裝置：",
            "Đang truyền APK sang thiết bị: ",
            "Μεταφορά APK στη συσκευή: ",
            "डिवाइस पर APK ट्रांसफ़र: ",
            "APK-ის მოწყობილობაზე გადატანა: ",
            "APK naar apparaat overzetten: ",
            "جارٍ نقل APK إلى الجهاز: ",
            "Transfiriendo APK al dispositivo: ",
        ],
    ),
    (
        "push_verified",
        [
            "APK 파일 크기 및 SHA-256 검증 완료",
            "APK size and SHA-256 verified",
            "Размер и SHA-256 APK проверены",
            "APKのサイズとSHA-256を確認しました",
            "已驗證 APK 大小與 SHA-256",
            "Đã xác minh kích thước và SHA-256 của APK",
            "Το μέγεθος και το SHA-256 του APK επαληθεύτηκαν",
            "APK आकार और SHA-256 सत्यापित",
            "APK-ის ზომა და SHA-256 დადასტურდა",
            "APK-grootte en SHA-256 gecontroleerd",
            "تم التحقق من حجم APK وبصمة SHA-256",
            "Verificados el tamaño y el SHA-256 del APK",
        ],
    ),
    (
        "install_start",
        [
            "전송 파일 검증 완료, Android에 APK 설치를 요청합니다.",
            "APK transfer verified; asking Android to install the package.",
            "Передача APK проверена; Android запускает установку.",
            "転送したAPKを確認しました。Androidにインストールを要求します。",
            "已驗證傳輸的 APK，正在要求 Android 安裝。",
            "Đã xác minh APK chuyển đến; đang yêu cầu Android cài đặt.",
            "Η μεταφορά APK επαληθεύτηκε· ζητείται εγκατάσταση από το Android.",
            "APK ट्रांसफ़र सत्यापित; Android इंस्टॉल शुरू कर रहा है।",
            "APK გადატანა შემოწმებულია; Android-ს ინსტალაცია ეთხოვა.",
            "APK-overdracht gecontroleerd; Android-installatie starten.",
            "تم التحقق من نقل APK؛ جارٍ طلب تثبيته من Android.",
            "Transferencia de APK verificada; solicitando la instalación a Android.",
        ],
    ),
    (
        "install_elapsed",
        [
            "설치 명령이 반환되었습니다. 경과 시간: ",
            "Installation command returned. Elapsed time: ",
            "Команда установки завершила работу. Время: ",
            "インストールコマンドが終了しました。経過時間: ",
            "安裝指令已返回。經過時間：",
            "Lệnh cài đặt đã trả về. Thời gian: ",
            "Η εντολή εγκατάστασης επέστρεψε. Χρόνος: ",
            "इंस्टॉल कमांड लौटी। बीता समय: ",
            "ინსტალაციის ბრძანება დასრულდა. გასული დრო: ",
            "Installatieopdracht is afgerond. Verstreken tijd: ",
            "اكتمل أمر التثبيت. الوقت المنقضي: ",
            "Comando de instalación finalizado. Tiempo transcurrido: ",
        ],
    ),
    (
        "install_done",
        [
            "설치 성공 및 패키지 등록 확인",
            "Installation succeeded and package registration verified",
            "Установка успешна, регистрация пакета проверена",
            "インストール成功、パッケージ登録を確認しました",
            "安裝成功且已確認套件註冊",
            "Cài đặt thành công và đã xác minh đăng ký gói",
            "Η εγκατάσταση πέτυχε και η εγγραφή πακέτου επαληθεύτηκε",
            "इंस्टॉल सफल; पैकेज पंजीकरण सत्यापित",
            "ინსტალაცია წარმატებულია, პაკეტის რეგისტრაცია დადასტურდა",
            "Installatie geslaagd en pakketregistratie gecontroleerd",
            "نجح التثبيت وتم التحقق من تسجيل الحزمة",
            "Instalación correcta y registro del paquete verificado",
        ],
    ),
    (
        "phase_ready",
        [
            "작업 준비 및 기기 확인",
            "Preparing and checking the device",
            "Подготовка и проверка устройства",
            "準備・端末確認",
            "準備與裝置檢查",
            "Chuẩn bị và kiểm tra thiết bị",
            "Προετοιμασία και έλεγχος συσκευής",
            "तैयारी और डिवाइस जाँच",
            "მომზადება და მოწყობილობის შემოწმება",
            "Voorbereiden en apparaat controleren",
            "الإعداد وفحص الجهاز",
            "Preparación y comprobación del dispositivo",
        ],
    ),
    (
        "phase_one",
        [
            "Google 서비스 준비 (1/4)",
            "Preparing Google services (1/4)",
            "Подготовка сервисов Google (1/4)",
            "Googleサービス準備（1/4）",
            "準備 Google 服務 (1/4)",
            "Chuẩn bị dịch vụ Google (1/4)",
            "Προετοιμασία υπηρεσιών Google (1/4)",
            "Google सेवाएँ तैयार की जा रही हैं (1/4)",
            "Google სერვისების მომზადება (1/4)",
            "Google-services voorbereiden (1/4)",
            "إعداد خدمات Google (1/4)",
            "Preparando servicios de Google (1/4)",
        ],
    ),
    (
        "phase_two",
        [
            "Google 서비스 APK 설치 (2/4)",
            "Installing Google service APKs (2/4)",
            "Установка APK сервисов Google (2/4)",
            "GoogleサービスAPKのインストール（2/4）",
            "安裝 Google 服務 APK (2/4)",
            "Cài đặt APK dịch vụ Google (2/4)",
            "Εγκατάσταση APK υπηρεσιών Google (2/4)",
            "Google सेवा APK इंस्टॉल हो रहे हैं (2/4)",
            "Google სერვისების APK ინსტალაცია (2/4)",
            "Google-service-APK's installeren (2/4)",
            "تثبيت ملفات APK لخدمات Google (2/4)",
            "Instalando APK de servicios de Google (2/4)",
        ],
    ),
    (
        "phase_three",
        [
            "Google 서비스 구성 (3/4)",
            "Configuring Google services (3/4)",
            "Настройка сервисов Google (3/4)",
            "Googleサービス設定（3/4）",
            "設定 Google 服務 (3/4)",
            "Cấu hình dịch vụ Google (3/4)",
            "Ρύθμιση υπηρεσιών Google (3/4)",
            "Google सेवाओं का कॉन्फ़िगरेशन (3/4)",
            "Google სერვისების კონფიგურაცია (3/4)",
            "Google-services configureren (3/4)",
            "تهيئة خدمات Google (3/4)",
            "Configurando servicios de Google (3/4)",
        ],
    ),
    (
        "phase_four",
        [
            "Google 서비스 최종 점검 (4/4)",
            "Final Google service checks (4/4)",
            "Финальная проверка сервисов Google (4/4)",
            "Googleサービスの最終確認（4/4）",
            "Google 服務最終檢查 (4/4)",
            "Kiểm tra cuối cùng dịch vụ Google (4/4)",
            "Τελικός έλεγχος υπηρεσιών Google (4/4)",
            "Google सेवाओं की अंतिम जाँच (4/4)",
            "Google სერვისების საბოლოო შემოწმება (4/4)",
            "Google-services controleren (4/4)",
            "الفحوص النهائية لخدمات Google (4/4)",
            "Comprobación final de servicios de Google (4/4)",
        ],
    ),
    (
        "phase_cleanup",
        [
            "기기 정리 및 검증",
            "Cleanup and verification",
            "Очистка и проверка",
            "後片付け・検証",
            "清理與驗證",
            "Dọn dẹp và xác minh",
            "Καθαρισμός και επαλήθευση",
            "सफ़ाई और सत्यापन",
            "გასუფთავება და შემოწმება",
            "Opschonen en controleren",
            "التنظيف والتحقق",
            "Limpieza y verificación",
        ],
    ),
    (
        "phase_done",
        [
            "작업 완료",
            "Operation complete",
            "Операция завершена",
            "作業完了",
            "作業完成",
            "Hoàn tất tác vụ",
            "Η λειτουργία ολοκληρώθηκε",
            "कार्य पूरा",
            "ოპერაცია დასრულდა",
            "Bewerking voltooid",
            "اكتملت العملية",
            "Operación finalizada",
        ],
    ),
    (
        "phase_failed",
        [
            "작업 중단됨",
            "Operation stopped",
            "Операция прервана",
            "作業停止",
            "作業已停止",
            "Tác vụ đã dừng",
            "Η λειτουργία διακόπηκε",
            "कार्य रुका",
            "ოპერაცია შეწყდა",
            "Bewerking gestopt",
            "توقفت العملية",
            "Operación detenida",
        ],
    ),
    (
        "cleanup_fail_retry",
        [
            "임시 폴더 삭제 재시도",
            "Temporary folder cleanup retry",
            "Повторная попытка удаления временной папки",
            "一時フォルダー削除を再試行",
            "重試刪除暫存資料夾",
            "Thử lại việc xóa thư mục tạm",
            "Επανάληψη καθαρισμού προσωρινού φακέλου",
            "अस्थायी फ़ोल्डर हटाने का पुनः प्रयास",
            "დროებითი საქაღალდის წაშლის ხელახალი ცდა",
            "Opnieuw proberen tijdelijke map te verwijderen",
            "إعادة محاولة حذف المجلد المؤقت",
            "Reintento de eliminación de la carpeta temporal",
        ],
    ),
    (
        "cleanup_fail",
        [
            "실패",
            "failed",
            "сбой",
            "失敗",
            "失敗",
            "thất bại",
            "απέτυχε",
            "विफल",
            "წარუმატებელია",
            "mislukt",
            "فشل",
            "falló",
        ],
    ),
    (
        "cleanup_started",
        [
            "[ADB] GBST 기기 임시 폴더 삭제 및 잔여 파일 검증 중...",
            "[ADB] Removing the device's temporary GBST folder and verifying cleanup...",
            "[ADB] Удаление временной папки GBST на устройстве и проверка очистки...",
            "[ADB] 端末のGBST一時フォルダーを削除し、残存ファイルを確認しています...",
            "[ADB] 正在刪除裝置上的 GBST 暫存資料夾並檢查殘留檔案...",
            "[ADB] Đang xóa thư mục GBST tạm trên thiết bị và kiểm tra tệp còn sót...",
            "[ADB] Διαγραφή προσωρινού φακέλου GBST και έλεγχος υπολειμμάτων...",
            "[ADB] डिवाइस का अस्थायी GBST फ़ोल्डर हटाकर बची फ़ाइलें जाँची जा रही हैं...",
            "[ADB] GBST-ის დროებითი საქაღალდის წაშლა და დარჩენილი ფაილების შემოწმება...",
            "[ADB] Tijdelijke GBST-map op het apparaat verwijderen en controleren...",
            "[ADB] جارٍ حذف مجلد GBST المؤقت والتحقق من الملفات المتبقية...",
            "[ADB] Eliminando la carpeta temporal GBST del dispositivo y comprobando los archivos restantes...",
        ],
    ),
    (
        "cleanup_verified",
        [
            "삭제 및 검증 완료",
            "removed and verified",
            "удалено, проверка завершена",
            "削除・検証完了",
            "已刪除並完成驗證",
            "đã xóa và xác minh",
            "διαγράφηκε και επαληθεύτηκε",
            "हटाया और सत्यापित किया",
            "წაიშალა და შემოწმდა",
            "verwijderd en gecontroleerd",
            "تم الحذف والتحقق",
            "eliminada y verificada",
        ],
    ),
];

pub fn operation_phrase(language: LanguageOption, key: &str) -> Option<&'static str> {
    let items = ENTRIES.iter().find(|(name, _)| *name == key)?.1;
    items.get(language as usize).copied()
}

fn package_message(language: LanguageOption, line: &str) -> Option<String> {
    let (package, detail) = line.split_once(": ")?;
    let message = match detail {
        "사용자 0 등록 상태 복구를 먼저 시도합니다." => {
            operation_phrase(language, "restore_user_try")?.to_string()
        }
        "사용자 0 패키지 등록 복원 완료" => {
            operation_phrase(language, "restore_user_done")?.to_string()
        }
        "설치 버전이 같거나 높아 기존 패키지를 재사용합니다." => {
            operation_phrase(language, "reuse_version")?.to_string()
        }
        "기기 전송 파일 크기 및 SHA-256 검증 완료" => {
            operation_phrase(language, "push_verified")?.to_string()
        }
        "APK 전송 검증 완료, Android 패키지 설치 명령을 실행합니다." => {
            operation_phrase(language, "install_start")?.to_string()
        }
        "설치 성공 및 패키지 등록 확인" => {
            operation_phrase(language, "install_done")?.to_string()
        }
        _ if detail.starts_with("기기 전송 진행률 ") && detail.ends_with('%') => {
            format!(
                "{}{}",
                operation_phrase(language, "push_progress")?,
                detail.trim_start_matches("기기 전송 진행률 ")
            )
        }
        _ if detail.ends_with(" 경로로 APK 전송을 시작합니다.") => {
            let path = detail.trim_end_matches(" 경로로 APK 전송을 시작합니다.");
            format!("{}{}", operation_phrase(language, "push_start")?, path)
        }
        _ if detail.starts_with("설치 명령 반환, 경과 ") && detail.ends_with("초") => {
            let seconds = detail
                .trim_start_matches("설치 명령 반환, 경과 ")
                .trim_end_matches("초");
            format!(
                "{}{}{}",
                operation_phrase(language, "install_elapsed")?,
                seconds,
                operation_phrase(language, "seconds")?
            )
        }
        _ => return None,
    };
    Some(format!("{package}: {message}"))
}

pub(super) fn translate_operation_log(language: LanguageOption, line: &str) -> Option<String> {
    if language == LanguageOption::Korean {
        return None;
    }
    match line {
        "[ADB] USB 디버깅 인증 필요: 태블릿의 잠금을 해제한 뒤 '이 컴퓨터에서 항상 허용'을 선택하고 '허용'을 누르세요." =>
        {
            return Some(format!("[ADB] {}", operation_phrase(language, "auth")?));
        }
        "[ADB] 이전 작업의 GBST 임시 파일이 남아 있는지 확인합니다." => {
            return Some(operation_phrase(language, "stale_check")?.into());
        }
        "[ADB] 이전 GBST 임시 파일 정리 확인 완료" => {
            return Some(operation_phrase(language, "stale_done")?.into());
        }
        "[ADB] GBST 기기 임시 폴더 삭제 및 잔여 파일 검증 중..." => {
            return Some(operation_phrase(language, "cleanup_started")?.into());
        }
        "[ADB] 작업 중단 후 GBST 임시 파일 정리 완료" => {
            return Some(operation_phrase(language, "cleanup_after_abort")?.into());
        }
        "[APK] APK 설치 중" => return Some(operation_phrase(language, "installing")?.into()),
        _ => {}
    }
    if line == "[ADB] /data/local/tmp/dwas_kr/GBST 삭제 및 검증 완료" {
        return Some(format!(
            "[ADB] /data/local/tmp/dwas_kr/GBST {}",
            operation_phrase(language, "cleanup_verified")?
        ));
    }
    if let Some(v) = line
        .strip_prefix("[ADB] 작업 안정화 대기: ")
        .and_then(|v| v.strip_suffix("초"))
    {
        return Some(format!(
            "{}{}{}",
            operation_phrase(language, "wait_prefix")?,
            v,
            operation_phrase(language, "seconds")?
        ));
    }
    if let Some(v) = line.strip_prefix("[Device] 사용자가 설정한 화면 꺼짐 값: ") {
        let val = if v == "화면 꺼짐 비활성화" || v == "사용 안 함" {
            operation_phrase(language, "timeout_off")?
        } else {
            v
        };
        return Some(format!(
            "{}{}",
            operation_phrase(language, "screen_timeout")?,
            val
        ));
    }
    if let Some(v) = line.strip_prefix("[Device] 사용자가 설정한 화면 밝기 값: ") {
        return Some(format!(
            "{}{}",
            operation_phrase(language, "brightness_label")?,
            v
        ));
    }
    if let Some(v) = line
        .strip_prefix("[Device] 사용자가 설정한 화면 밝기 값(")
        .and_then(|v| v.strip_suffix(")으로 복원합니다."))
    {
        return Some(format!(
            "{}{}",
            operation_phrase(language, "brightness_restore")?,
            v
        ));
    }
    if let Some(v) = line
        .strip_prefix("[Device] 사용자 설정 ")
        .and_then(|v| v.strip_suffix(" 복원 완료"))
    {
        return Some(format!(
            "{}{}",
            operation_phrase(language, "settings_restore")?,
            v
        ));
    }
    if let Some(v) = line.strip_prefix("[ADB] 임시 폴더 삭제 재시도 ") {
        if let Some((attempt, error)) = v.split_once(" 실패: ") {
            return Some(format!(
                "[ADB] {} {attempt} {}: {error}",
                operation_phrase(language, "cleanup_fail_retry")?,
                operation_phrase(language, "cleanup_fail")?
            ));
        }
    }
    if let Some(v) = line.strip_prefix("[APK] ") {
        return package_message(language, v).map(|value| format!("[APK] {value}"));
    }
    if let Some(v) = line.strip_prefix("[ADB] ") {
        return package_message(language, v).map(|value| format!("[ADB] {value}"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn user_success_log_samples_translate_for_all_supported_languages() {
        let lines = [
            "[ADB] USB 디버깅 인증 필요: 태블릿의 잠금을 해제한 뒤 '이 컴퓨터에서 항상 허용'을 선택하고 '허용'을 누르세요.",
            "[ADB] 이전 작업의 GBST 임시 파일이 남아 있는지 확인합니다.",
            "[ADB] 이전 GBST 임시 파일 정리 확인 완료",
            "[Device] 사용자가 설정한 화면 꺼짐 값: 화면 꺼짐 비활성화",
            "[Device] 사용자가 설정한 화면 밝기 값: 53",
            "[ADB] 작업 안정화 대기: 2초",
            "[APK] APK 설치 중",
            "[ADB] com.google.android.partnersetup: 사용자 0 등록 상태 복구를 먼저 시도합니다.",
            "[ADB] com.google.android.partnersetup: 사용자 0 패키지 등록 복원 완료",
            "[APK] com.google.android.gms: 설치 버전이 같거나 높아 기존 패키지를 재사용합니다.",
            "[APK] com.google.android.gms: /data/local/tmp/dwas_kr/GBST/com.google.android.gms.apk 경로로 APK 전송을 시작합니다.",
            "[APK] com.google.android.gms: 기기 전송 진행률 70%",
            "[APK] com.google.android.gms: 기기 전송 파일 크기 및 SHA-256 검증 완료",
            "[APK] com.google.android.gms: APK 전송 검증 완료, Android 패키지 설치 명령을 실행합니다.",
            "[APK] com.google.android.gms: 설치 명령 반환, 경과 17초",
            "[APK] com.google.android.gms: 설치 성공 및 패키지 등록 확인",
            "[Device] 사용자가 설정한 화면 밝기 값(53)으로 복원합니다.",
            "[Device] 사용자 설정 user_rotation 복원 완료",
            "[ADB] GBST 기기 임시 폴더 삭제 및 잔여 파일 검증 중...",
            "[ADB] /data/local/tmp/dwas_kr/GBST 삭제 및 검증 완료",
            "[ADB] 작업 중단 후 GBST 임시 파일 정리 완료",
        ];
        for language in LanguageOption::ALL {
            if language == LanguageOption::Korean {
                continue;
            }
            for line in lines {
                let translated = translate_operation_log(language, line)
                    .unwrap_or_else(|| panic!("untranslated {}: {line}", language.code()));
                assert!(
                    !translated.chars().any(|c| ('가'..='힣').contains(&c)),
                    "{}: {translated}",
                    language.code()
                );
            }
        }
    }
}
