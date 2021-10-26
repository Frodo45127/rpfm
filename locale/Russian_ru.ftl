### Localization for RPFM-UI - Russian
## Translated by im_mortal


## These two need to be changed for special builds, so they go first.

title_only_for_the_brave = Для самых смелых
message_only_for_the_brave =
    <p>
        Эта версия помечена как «Для самых смелых».
        Это означает, что эта бета-версия содержит крайне нестабильные или неопробованные возможности,
        которые могут вызвать проблемы во время использования.
        В то же время, перед Вами уникальная возможность попробовать новшества раньше других.
    </p>

    <p>
        Если Вы не готовы к такому риску, смените канал обновлений на "Стабильный"
        и проверьте наличие обновлений. Эта процедура должна откатить используемую
        версию { -app } к последней стабильной версии.
    </p>

    <p>
        В версиях «Для самых смелых» настоятельно рекомендуется делать регулярные резервные копии модов, прежде чем использовать эту версию { -app } с ними. Ознакомьтесь с нестабильными функциями этой версии в списке ниже:
    </p>

    <ul>
        <li>
            Редактор RigidModel: был обновлён, но едва ли тестировался.
            Если Вы собираетесь редактировать моды с моделями RigidModel и не хотите случайно их повредить, можете отключить его в настройках.
        </li>
        <li>
            Редактор ESF: получил крайне ограниченное тестирование. Если вы собираетесь редактировать моды с файлами
            ESF/CCD/SAVE и не хотите случайно их повредить, можете отключить его в настройках.
        </li>
    </ul>

    <p>Примечания к редактору RigidModel:</p>
    <ul>
        <li>
            Некоторые кнопки могут выглядеть нерабочими.
            Это будет исправлено в будущем обновлении..
        </li>
    </ul>

    <p>Примечания к редактору ESF:</p>
    <ul>
        <li>Поддерживает файлы ESF только одного формата (может не открывать слишком старые файлы).</li>
        <li>Не поддерживает импорт/экспорт.</li>
        <li>Не поддерживает редактирование сжатых узлов.</li>
        <li>Текстовые элементы "Label" не функциональны. Игнорируйте их.</li>
        <li>
            Некоторые числовые поля могут принимать большие значения, чем положено.
            Это лишь ошибка интерфейса. «Под капотом» значения проверяются и исправляются по мере необходимости.
        </li>
    </ul>

## Translation variables

-app =
    { $full ->
       *[false] RPFM
        [true] Rusted PackFile Manager
    }
    .gender = masculine
    .startsWith = vowel

-packfile =
    { $capitalization ->
       *[capitalized] Pack-файл{ $case ->
           *[nominative] { $number ->
               *[one] { "" }
                [few] а
                [many] ов
                [other] ы
            }
            [genitive] { $number ->
               *[one] а
                [few] ов
                [many] ов
                [other] ов
            }
            [dative] { $number ->
               *[one] у
                [few] ам
                [many] ам
                [other] ам
            }
            [accusative] { $number ->
               *[one] { "" }
                [few] а
                [many] ов
                [other] ы
            }
            [instrumental] { $number ->
               *[one] ом
                [few] ами
                [many] ами
                [other] ами
            }
            [prepositional] { $number ->
               *[one] е
                [few] ах
                [many] ах
                [other] ах
            }
        }
        [upper] PACK-ФАЙЛ{ $case ->
           *[nominative] { $number ->
               *[one] { "" }
                [few] А
                [many] ОВ
                [other] Ы
            }
            [genitive] { $number ->
               *[one] А
                [few] ОВ
                [many] ОВ
                [other] ОВ
            }
            [dative] { $number ->
               *[one] У
                [few] АМ
                [many] аМ
                [other] АМ
            }
            [accusative] { $number ->
               *[one] { "" }
                [few] А
                [many] ОВ
                [other] Ы
            }
            [instrumental] { $number ->
               *[one] ОМ
                [few] АМИ
                [many] АМИ
                [other] АМИ
            }
            [prepositional] { $number ->
               *[one] Е
                [few] АХ
                [many] АХ
                [other] АХ
            }
        }
        [lower] pack-файл{ $case ->
           *[nominative] { $number ->
               *[one] { "" }
                [few] а
                [many] ов
                [other] ы
            }
            [genitive] { $number ->
               *[one] а
                [few] ов
                [many] ов
                [other] ов
            }
            [dative] { $number ->
               *[one] у
                [few] ам
                [many] ам
                [other] ам
            }
            [accusative] { $number ->
               *[one] { "" }
                [few] а
                [many] ов
                [other] ы
            }
            [instrumental] { $number ->
               *[one] ом
                [few] ами
                [many] ами
                [other] ами
            }
            [prepositional] { $number ->
               *[one] е
                [few] ах
                [many] ах
                [other] ах
            }
        }
    }
    .gender = masculine
    .startsWith = consonant

-packedfile =
    { $case ->
       *[nominative] { $number ->
           *[one] { $capitalization ->
               *[capitalized] Запакованный файл
                [upper] ЗАПАКОВАННЫЙ ФАЙЛ
                [lower] запакованный файл
            }
            [few] { $capitalization ->
               *[capitalized] Запакованных файла
                [upper] ЗАПАКОВАННЫХ ФАЙЛА
                [lower] запакованных файла
            }
            [many] { $capitalization ->
               *[capitalized] Запакованных файлов
                [upper] ЗАПАКОВАННЫХ ФАЙЛОВ
                [lower] запакованных файлов
            }
            [other] { $capitalization ->
               *[capitalized] Запакованные файлы
                [upper] ЗАПАКОВАННЫЕ ФАЙЛЫ
                [lower] запакованные файлы
            }
        }
        [genitive] { $number ->
           *[one] { $capitalization ->
               *[capitalized] Запакованного файла
                [upper] ЗАПАКОВАННОГО ФАЙЛА
                [lower] запакованного файла
            }
            [few] { $capitalization ->
               *[capitalized] Запакованных файлов
                [upper] ЗАПАКОВАННЫХ ФАЙЛОВ
                [lower] запакованных файлов
            }
            [many] { $capitalization ->
               *[capitalized] Запакованных файлов
                [upper] ЗАПАКОВАННЫХ ФАЙЛОВ
                [lower] запакованных файлов
            }
            [other] { $capitalization ->
               *[capitalized] Запакованных файлов
                [upper] ЗАПАКОВАННЫХ ФАЙЛОВ
                [lower] запакованных файлов
            }
        }
        [dative] { $number ->
           *[one] { $capitalization ->
               *[capitalized] Запакованному файлу
                [upper] ЗАПАКОВАННОМУ ФАЙЛУ
                [lower] запакованному файлу
            }
            [few] { $capitalization ->
               *[capitalized] Запакованным файлам
                [upper] ЗАПАКОВАННЫМ ФАЙЛАМ
                [lower] запакованным файлам
            }
            [many] { $capitalization ->
               *[capitalized] Запакованным файлам
                [upper] ЗАПАКОВАННЫМ ФАЙЛАМ
                [lower] запакованным файлам
            }
            [other] { $capitalization ->
               *[capitalized] Запакованным файлам
                [upper] ЗАПАКОВАННЫМ ФАЙЛАМ
                [lower] запакованным файлам
            }
        }
        [accusative] { $number ->
           *[one] { $capitalization ->
               *[capitalized] Запакованный файл
                [upper] ЗАПАКОВАННЫЙ ФАЙЛ
                [lower] запакованный файл
            }
            [few] { $capitalization ->
               *[capitalized] Запакованным файлам
                [upper] ЗАПАКОВАННЫМ ФАЙЛАМ
                [lower] запакованным файлам
            }
            [many] { $capitalization ->
               *[capitalized] Запакованным файлам
                [upper] ЗАПАКОВАННЫМ ФАЙЛАМ
                [lower] запакованным файлам
            }
            [other] { $capitalization ->
               *[capitalized] Запакованные файлы
                [upper] ЗАПАКОВАННЫЕ ФАЙЛЫ
                [lower] запакованные файлы
            }
        }
        [instrumental] { $number ->
           *[one] { $capitalization ->
               *[capitalized] Запакованным файлом
                [upper] ЗАПАКОВАННЫМ ФАЙЛОМ
                [lower] запакованным файлом
            }
            [few] { $capitalization ->
               *[capitalized] Запакованными файлами
                [upper] ЗАПАКОВАННЫМИ ФАЙЛАМИ
                [lower] запакованными файлами
            }
            [many] { $capitalization ->
               *[capitalized] Запакованными файлами
                [upper] ЗАПАКОВАННЫМИ ФАЙЛАМИ
                [lower] запакованными файлами
            }
            [other] { $capitalization ->
               *[capitalized] Запакованными файлами
                [upper] ЗАПАКОВАННЫМИ ФАЙЛАМИ
                [lower] запакованными файлами
            }
        }
        [prepositional] { $number ->
           *[one] { $capitalization ->
               *[capitalized] Запакованном файле
                [upper] ЗАПАКОВАННОМ ФАЙЛЕ
                [lower] запакованном файле
            }
            [few] { $capitalization ->
               *[capitalized] Запакованных файлах
                [upper] ЗАПАКОВАННЫХ ФАЙЛАХ
                [lower] запакованных файлах
            }
            [many] { $capitalization ->
               *[capitalized] Запакованных файлах
                [upper] ЗАПАКОВАННЫХ ФАЙЛАХ
                [lower] запакованных файлах
            }
            [other] { $capitalization ->
               *[capitalized] Запакованных файлах
                [upper] ЗАПАКОВАННЫХ ФАЙЛАХ
                [lower] запакованных файлах
            }
        }
    }
    .gender = masculine
    .startsWith = consonant

-animpack =
    { $capitalization ->
       *[capitalized] AnimPack
        [upper] ANIMPACK
        [lower] animpack
    }{ $number ->
       *[singular] { "" }
        [plural] s
    }
    .gender = masculine

-CA = CA
    .gender = feminine

-mymod = MyMod
    .gender = masculine

-AssemblyKit =
    { $capitalization ->
       *[capitalized] Assembly Kit
        [upper] ASSEMBLY KIT
        [lower] assembly kit
    }
    .gender = masculine

-Data =
    { $capitalization ->
       *[capitalized] Data
        [upper] DATA
        [lower] data
    }
    .number = other
    .gender = other

-db =
    { $capitalization ->
       *[capitalized] Таблиц{ $case ->
           *[nominative] { $number ->
               *[one] а
                [few] ы
                [many] { "" }
                [other] ы
            }
            [genitive] { $number ->
               *[one] ы
                [few] { "" }
                [many] { "" }
                [other] { "" }
            }
            [dative] { $number ->
               *[one] е
                [few] ам
                [many] ам
                [other] ам
            }
            [accusative] { $number ->
               *[one] у
                [few] ы
                [many] { "" }
                [other] ы
            }
            [instrumental] { $number ->
               *[one] ей
                [few] ами
                [many] ами
                [other] ами
            }
            [prepositional] { $number ->
               *[one] е
                [few] ах
                [many] ах
                [other] ах
            }
        }
        [upper] ТАБЛИЦ{ $case ->
           *[nominative] { $number ->
               *[one] А
                [few] Ы
                [many] { "" }
                [other] Ы
            }
            [genitive] { $number ->
               *[one] Ы
                [few] { "" }
                [many] { "" }
                [other] { "" }
            }
            [dative] { $number ->
               *[one] Е
                [few] АМ
                [many] АМ
                [other] АМ
            }
            [accusative] { $number ->
               *[one] У
                [few] Ы
                [many] { "" }
                [other] Ы
            }
            [instrumental] { $number ->
               *[one] ЕЙ
                [few] АМИ
                [many] АМИ
                [other] ами
            }
            [prepositional] { $number ->
               *[one] Е
                [few] АХ
                [many] АХ
                [other] АХ
            }
        }
        [lower] таблиц{ $case ->
           *[nominative] { $number ->
               *[one] а
                [few] ы
                [many] { "" }
                [other] ы
            }
            [genitive] { $number ->
               *[one] ы
                [few] { "" }
                [many] { "" }
                [other] { "" }
            }
            [dative] { $number ->
               *[one] е
                [few] ам
                [many] ам
                [other] ам
            }
            [accusative] { $number ->
               *[one] у
                [few] ы
                [many] { "" }
                [other] ы
            }
            [instrumental] { $number ->
               *[one] ей
                [few] ами
                [many] ами
                [other] ами
            }
            [prepositional] { $number ->
               *[one] е
                [few] ах
                [many] ах
                [other] ах
            }
        }
    }{ $includeOriginalText ->
        [true]  ({ $capitalization ->
           *[capitalized] DB
            [lower] db
        })
       *[false] 
    }
    .gender = feminine
    .startsWith = consonant

-loc =
    { $capitalization ->
       *[capitalized] Файл{ $case ->
           *[nominative] { $number ->
               *[one] 
                [few] а
                [many] ов
                [other] ы
            }
            [genitive] { $number ->
               *[one] а
                [few] ов
                [many] ов
                [other] ов
            }
            [dative] { $number ->
               *[one] у
                [few] ам
                [many] ам
                [other] ам
            }
            [accusative] { $number ->
               *[one] { "" }
                [few] а
                [many] ов
                [other] ы
            }
            [instrumental] { $number ->
               *[one] ом
                [few] ами
                [many] ами
                [other] ами
            }
            [prepositional] { $number ->
               *[one] е
                [few] ах
                [many] ах
                [other] ах
            }
        } локализации
        [upper] ФАЙЛ{ $case ->
           *[nominative] { $number ->
               *[one] { "" }
                [few] А
                [many] ОВ
                [other] Ы
            }
            [genitive] { $number ->
               *[one] А
                [few] ОВ
                [many] ОВ
                [other] ОВ
            }
            [dative] { $number ->
               *[one] У
                [few] АМ
                [many] аМ
                [other] АМ
            }
            [accusative] { $number ->
               *[one] { "" }
                [few] А
                [many] ОВ
                [other] Ы
            }
            [instrumental] { $number ->
               *[one] ОМ
                [few] АМИ
                [many] АМИ
                [other] АМИ
            }
            [prepositional] { $number ->
               *[one] Е
                [few] АХ
                [many] АХ
                [other] АХ
            }
        } ЛОКАЛИЗАЦИИ
        [lower] файл{ $case ->
           *[nominative] { $number ->
               *[one] { "" }
                [few] а
                [many] ов
                [other] ы
            }
            [genitive] { $number ->
               *[one] а
                [few] ов
                [many] ов
                [other] ов
            }
            [dative] { $number ->
               *[one] у
                [few] ам
                [many] ам
                [other] ам
            }
            [accusative] { $number ->
               *[one] { "" }
                [few] а
                [many] ов
                [other] ы
            }
            [instrumental] { $number ->
               *[one] ом
                [few] ами
                [many] ами
                [other] ами
            }
            [prepositional] { $number ->
               *[one] е
                [few] ах
                [many] ах
                [other] ах
            }
        } локализации
    }{ $includeOriginalText ->
        [true]  ({ $capitalization ->
           *[capitalized] LOC
            [upper] loc
            [lower] loc
        })
       *[false] 
    }
    .gender = masculine
    .startsWith = consonant

-schema =
    { $capitalization ->
       *[capitalized] Схем{ $case ->
           *[nominative] { $number ->
               *[one] а
                [few] ы
                [many] { "" }
                [other] ы
            }
            [genitive] { $number ->
               *[one] ы
                [few] { "" }
                [many] { "" }
                [other] { "" }
           }
            [dative] { $number ->
               *[one] е
                [few] ам
                [many] ам
                [other] ам
            }
            [accusative] { $number ->
               *[one] у
                [few] ы
                [many] { "" }
                [other] ы
            }
            [instrumental] { $number ->
               *[one] ой
                [few] ами
                [many] ами
                [other] ами
            }
            [prepositional] { $number ->
               *[one] е
                [few] ах
                [many] ах
                [other] ах
            }
        }
        [upper] СХЕМ{ $case ->
           *[nominative] { $number ->
               *[one] А
                [few] Ы
                [many] { "" }
                [other] Ы
            }
            [genitive] { $number ->
               *[one] Ы
                [few] { "" }
                [many] { "" }
                [other] { "" }
           }
            [dative] { $number ->
               *[one] Е
                [few] АМ
                [many] АМ
                [other] АМ
            }
            [accusative] { $number ->
               *[one] У
                [few] Ы
                [many] { "" }
                [other] Ы
            }
            [instrumental] { $number ->
               *[one] ОЙ
                [few] АМИ
                [many] АМИ
                [other] АМИ
            }
            [prepositional] { $number ->
               *[one] Е
                [few] АХ
                [many] АХ
                [other] АХ
            }
        }
        [lower] схем{ $case ->
           *[nominative] { $number ->
               *[one] а
                [few] ы
                [many] { "" }
                [other] ы
            }
            [genitive] { $number ->
               *[one] ы
                [few] { "" }
                [many] { "" }
                [other] { "" }
           }
            [dative] { $number ->
               *[one] е
                [few] ам
                [many] ам
                [other] ам
            }
            [accusative] { $number ->
               *[one] у
                [few] ы
                [many] { "" }
                [other] ы
            }
            [instrumental] { $number ->
               *[one] ой
                [few] ами
                [many] ами
                [other] ами
            }
            [prepositional] { $number ->
               *[one] е
                [few] ах
                [many] ах
                [other] ах
            }
        }
    }
    .gender = feminine
    .startsWith = consonant

-game_selected =
    { $case ->
       *[nominative] { $number ->
           *[one] { $capitalization ->
               *[capitalized] Выбранная игра
                [upper] ВЫБРАННАЯ ИГРА
                [lower] выбранная игра
            }
            [few] { $capitalization ->
               *[capitalized] Выбранные игры
                [upper] ВЫБРАННЫЕ ИГРЫ
                [lower] выбранные игры
            }
            [many] { $capitalization ->
               *[capitalized] Выбранных игр
                [upper] ВЫБРАННЫХ ИГРЫ
                [lower] выбранных игры
            }
            [other] { $capitalization ->
               *[capitalized] Выбранные игры
                [upper] ВЫБРАННЫЕ ИГРЫ
                [lower] выбранные игры
            }
        }
        [genitive] { $number ->
           *[one] { $capitalization ->
               *[capitalized] Выбранной игры
                [upper] ВЫБРАННОЙ ИГРЫ
                [lower] выбранной игры
            }
            [few] { $capitalization ->
               *[capitalized] Выбранных игр
                [upper] ВЫБРАННЫХ ИГР
                [lower] выбранных игр
            }
            [many] { $capitalization ->
               *[capitalized] Выбранных игр
                [upper] ВЫБРАННЫХ ИГР
                [lower] выбранных игр
            }
            [other] { $capitalization ->
               *[capitalized] Выбранных игр
                [upper] ВЫБРАННЫХ ИГР
                [lower] выбранных игр
            }
        }
        [dative] { $number ->
           *[one] { $capitalization ->
               *[capitalized] Выбранной игре
                [upper] ВЫБРАННОЙ ИГРЕ
                [lower] выбранной игре
            }
            [few] { $capitalization ->
               *[capitalized] Выбранным играм
                [upper] ВЫБРАННЫМ ИГРАМ
                [lower] выбранным играм
            }
            [many] { $capitalization ->
               *[capitalized] Выбранным играм
                [upper] ВЫБРАННЫМ ИГРАМ
                [lower] выбранным играм
            }
            [other] { $capitalization ->
               *[capitalized] Выбранным играм
                [upper] ВЫБРАННЫМ ИГРАМ
                [lower] выбранным играм
            }
        }
        [accusative] { $number ->
           *[one] { $capitalization ->
               *[capitalized] Выбранную игру
                [upper] ВЫБРАННУЮ ИГРУ
                [lower] выбранную игру
            }
            [few] { $capitalization ->
               *[capitalized] Выбранные игры
                [upper] ВЫБРАННЫЕ ИГРЫ
                [lower] выбранные игры
            }
            [many] { $capitalization ->
               *[capitalized] Выбранных игр
                [upper] ВЫБРАННЫХ ИГР
                [lower] выбранных игр
            }
            [other] { $capitalization ->
               *[capitalized] Выбранные игры
                [upper] ВЫБРАННЫЕ ИГРЫ
                [lower] выбранные игры
            }
        }
        [instrumental] { $number ->
           *[one] { $capitalization ->
               *[capitalized] Выбранной игрой
                [upper] ВЫБРАННОЙ ИГРОЙ
                [lower] выбранной игрой
            }
            [few] { $capitalization ->
               *[capitalized] Выбранными играми
                [upper] ВЫБРАННЫМИ ИГРАМИ
                [lower] выбранными играми
            }
            [many] { $capitalization ->
               *[capitalized] Выбранными играми
                [upper] ВЫБРАННЫМИ ИГРАМИ
                [lower] выбранными играми
            }
            [other] { $capitalization ->
               *[capitalized] Выбранными играми
                [upper] ВЫБРАННЫМИ ИГРАМИ
                [lower] выбранными играми
            }
        }
        [prepositional] { $number ->
           *[one] { $capitalization ->
               *[capitalized] Выбранной игре
                [upper] ВЫБРАННОЙ ИГРЕ
                [lower] выбранной игре
            }
            [few] { $capitalization ->
               *[capitalized] Выбранных играх
                [upper] ВЫБРАННЫХ ИГРАХ
                [lower] выбранных играх
            }
            [many] { $capitalization ->
               *[capitalized] Выбранных играх
                [upper] ВЫБРАННЫХ ИГРАХ
                [lower] выбранных играх
            }
            [other] { $capitalization ->
               *[capitalized] Выбранных играх
                [upper] ВЫБРАННЫХ ИГРАХ
                [lower] выбранных играх
            }
        }
    }
    .gender = feminine
    .startsWith = consonant

-local-packfile_type_boot =
    { $accessKey ->
       *[false] { $capitalization ->
           *[capitalized] Boot
            [upper] BOOT
            [lower] boot
        }
        [true] { $capitalization ->
           *[capitalized] &Boot
            [upper] &BOOT
            [lower] &boot
        }
    }
    .gender = masculine
-local-packfile_type_patch =
    { $accessKey ->
       *[false] { $capitalization ->
           *[capitalized] Patch
            [upper] PATCH
            [lower] patch
        }
        [true] { $capitalization ->
           *[capitalized] &Patch
            [upper] &PATCH
            [lower] &patch
        }
    }
    .gender = masculine
-local-packfile_type_release =
    { $accessKey ->
       *[false] { $capitalization ->
           *[capitalized] Release
            [upper] RELEASE
            [lower] release
        }
        [true] { $capitalization ->
           *[capitalized] &Release
            [upper] &RELEASE
            [lower] &release
        }
    }
    .gender = masculine
-local-packfile_type_mod =
    { $accessKey ->
       *[false] { $capitalization ->
           *[capitalized] Mod
            [upper] MOD
            [lower] mod
        }
        [true] { $capitalization ->
           *[capitalized] &Mod
            [upper] &MOD
            [lower] &mod
        }
    }
    .gender = masculine
-local-packfile_type_movie =
    { $accessKey ->
       *[false] { $capitalization ->
           *[capitalized] Movie
            [upper] MOVIE
            [lower] movie
        }
        [true] { $capitalization ->
           *[capitalized] Mo&vie
            [upper] MO&VIE
            [lower] mo&vie
        }
    }
    .gender = masculine
-local-packfile_type_other =
    { $accessKey ->
       *[false] { $capitalization ->
           *[capitalized] Другой
            [upper] ДРУГОЙ
            [lower] другой
        }
        [true] { $capitalization ->
           *[capitalized] &Другой
            [upper] &ДРУГОЙ
            [lower] &другой
        }
    }{ $includeOriginalText ->
       *[false] { "" }
        [true]  ({ $capitalization ->
           *[capitalized] Other
            [upper] OTHER
            [lower] other
        })
    }
    .gender = masculine

## General Localization

gen_loc_accept = Принять
gen_loc_create = Создать
gen_loc_packedfile = { -packedfile(capitalization: "capitalized")  }
gen_loc_packfile = { -packfile(capitalization: "capitalized")  }
gen_loc_packfile_contents = Содержимое { -packfile(capitalization: "capitalized")  }

gen_loc_column = Столбец
gen_loc_row = Строка
gen_loc_match = Совпадение
gen_loc_length = Длина

trololol = queek_headtaker_yes_yes

## mod.rs localization

## Menu Bar

menu_bar_packfile = &{ -packfile(capitalization: "capitalized")  }
menu_bar_view = &Вид
menu_bar_mymod = &{ -mymod }
menu_bar_game_selected = Выбранная и&гра
menu_bar_special_stuff = До&полн.
menu_bar_templates = &Шаблоны
menu_bar_about = &О программе
menu_bar_debug = Отла&дка

## PackFile Menu

new_packfile = &Новый { -packfile(capitalization: "capitalized") }
open_packfile = &Открыть { -packfile(capitalization: "capitalized") }
save_packfile = &Сохранить { -packfile(capitalization: "capitalized") }
save_packfile_as = Сохранить { -packfile(capitalization: "capitalized") } &как…
packfile_install = &Установить
packfile_uninstall = Уда&лить
load_all_ca_packfiles = &Загрузить все { -packfile(number: "other") } от { -CA(case: "genitive") }
preferences = Нас&тройки
quit = &Выход
open_recent = Недавние
open_from_content = Открыть из содержимого
open_from_data = Открыть из папки Data
change_packfile_type = С&менить тип { -packfile(case: "genitive") }

## Change Packfile Type Menu

packfile_type_boot = { -local-packfile_type_boot(accessKey: "true") }
packfile_type_release = { -local-packfile_type_release(accessKey: "true") }
packfile_type_patch = { -local-packfile_type_patch(accessKey: "true") }
packfile_type_mod = { -local-packfile_type_mod(accessKey: "true") }
packfile_type_movie = { -local-packfile_type_movie(accessKey: "true") }
packfile_type_other = { -local-packfile_type_other(accessKey: "true") }

change_packfile_type_header_is_extended = Расширенный &заголовок
change_packfile_type_index_includes_timestamp = &Индекс с временно́й меткой
change_packfile_type_index_is_encrypted = Индекс за&шифрован
change_packfile_type_data_is_encrypted = &Данные зашифрованы
change_packfile_type_data_is_compressed = Данные с&жаты

## MyMod Menu

mymod_new =
    { -mymod.gender ->
       *[masculine] &Новый
        [feminine] &Новая
        [other] &Новое
    } { -mymod }

mymod_delete_selected =
    У&далить { -mymod.gender ->
       *[masculine] выбранный
        [feminine] выбранную
        [other] выбранное
    } { -mymod }

mymod_import = Импорт
mymod_export = Экспорт

mymod_name = Имя мода:
mymod_name_default = Например: one_ring_for_me
mymod_game = Мод для игры:

## View Menu

view_toggle_packfile_contents = Содержимое &{ -packfile(case: "genitive", capitalization: "capitalized") }
view_toggle_global_search_panel = Окно глобального поиска
view_toggle_diagnostics_panel = Окно диагностики
view_toggle_dependencies_panel = Окно зависимостей

## Game Selected Menu

game_selected_launch_game = Запустить { -game_selected(case: "accusative", capitalization: "lower") }
game_selected_open_game_data_folder = Открыть папку { -Data } { -game_selected(case: "genitive", capitalization: "lower") }
game_selected_open_game_assembly_kit_folder = Открыть папку { -AssemblyKit }
    { -game_selected(case: "genitive", capitalization: "lower") }
game_selected_open_config_folder = Открыть папку настроек { -app }

## Special Stuff

special_stuff_optimize_packfile = &Оптимизировать { -packfile(capitalization: "capitalized") }
special_stuff_patch_siege_ai = &Применить патч осадного ИИ
special_stuff_select_ak_folder = Выбрать папку { -AssemblyKit }
special_stuff_select_raw_db_folder = Выбрать папку с исходными { -db(case: "instrumental", number: "many") }

## Templates Menu
templates_open_custom_templates_folder = Открыть папку пользовательских шаблонов
templates_open_official_templates_folder = Открыть папку официальных шаблонов
templates_save_packfile_to_template = Сохранить { -packfile(capitalization: "lower") } в шаблон
templates_load_custom_template_to_packfile = Загрузить пользовательские шаблоны в { -packfile(capitalization: "capitalized") }
templates_load_official_template_to_packfile = Загрузить официальные шаблоны в { -packfile(capitalization: "capitalized") }

## About Menu

about_about_qt = О &Qt
about_about_rpfm =
    { -app.startsWith ->
        [vowel] Об
       *[consonant] О
    } &{ -app }
about_open_manual = Открыть &руководство
about_patreon_link = Поддержать на &Patreon
about_check_updates = Проверить &обновления
about_check_schema_updates = Проверить обновления &{ -schema(case: "genitive", number: "many", capitalization: "lower") }

## Debug Menu

update_current_schema_from_asskit =
    Обновить загруженн{ -schema.gender ->
        [masculine] ого
        [feminine] ую
       *[other] ое
    } { -schema(case: "accusative", capitalization: "lower", number: "one") } с помощью { -AssemblyKit(case: "instrumental") }
generate_schema_diff = Создать diff { -schema(case: "genitive", capitalization: "lower") }

## app_ui_extra.rs

## Update Stuff

-Checker = Модуль проверки
    .gender = masculine

update_checker = Обновить { -Checker(case: "accusative") }
update_schema_checker = Обновить { -Checker(case: "accusative") } { -schema(case: "genitive") }
update_template_checker = Обновить { -Checker(case: "accusative") } шаблонов
update_searching = Поиск обновлений…
update_button = &Обновить
update_in_prog =
    <p>Не закрывайте это окно, пока идёт загрузка обновлений…</p>
    <p>Этот процесс может занять некоторое время.</p>
update_no_local_schema =
    <p>
    Локальных { -schema(case: "genitive", capitalization: "lower", number: "many") } не найдено.
    Желаете ли Вы загрузить последние { -schema(case: "nominative", capitalization: "lower", number: "other") }?
    </p>
    <p>
        <b>ВНИМАНИЕ:</b>
        { -schema(case: "nominative", number: "other") }
        необходимы для открытия { -db(case: "genitive", number: "many") },
        { -loc(case: "genitive", number: "many") } и прочих { -packedfile(number: "many") }.
    </p>
update_no_local_template =
    <p>
    Локальных шаблонов не найдено. Желаете ли загрузить последние шаблоны?
    </p>
    <p>
        <b>ВНИМАНИЕ:</b>
        Шаблоны полезны при создании новых модов в два клика.
    </p>

## Folder Dialogues

new_folder_default = new_folder
new_folder = Новая папка

## PackedFile Dialogues

new_file_default = new_file
new_db_file =
    { -packedfile.gender ->
        [masculine] Новый
        [feminine] Новая
       *[other] Новое
    } { -packedfile(capitalization: "lower") } { -db(case: "genitive", capitalization: "lower") }
new_loc_file =
    { -packedfile.gender ->
        [masculine] Новый
        [feminine] Новая
       *[other] Новое
    } { -packedfile(capitalization: "lower") } локализации
new_txt_file =
    { -packedfile.gender ->
        [masculine] Новый текстовый
        [feminine] Новая текстовая
       *[other] Новое текстовое
    } { -packedfile(capitalization: "lower") }
new_animpack_file =
    { -animpack.gender ->
        [masculine] Новый
        [feminine] Новая
       *[other] Новое
    } { -animpack(capitalization: "lower") }
new_packedfile_name =
    Имя { -animpack.gender ->
        [feminine] новой
       *[other] нового
    } { -animpack(case: "genitive") }

packedfile_filter = Начните набирать текст здесь, чтобы отфильтровать таблицы в списке. Также работает и с регулярными выражениями!

merge_tables = Объединить таблицы
merge_tables_new_name = Укажите имя нового файла.
merge_tables_delete_option = Удалить исходные таблицы

## External FileDialog

open_packfiles = Открыть { -packedfile(number: "many") }

## tips.rs

## PackFile menu tips

tt_packfile_new_packfile = Создает новый { -packfile(capitalization: "lower") } и открывает его. Не забудьте сохранить его позднее, если хотите вернуться к нему.
tt_packfile_open_packfile = Открыть существующий { -packfile(capitalization: "lower") } или несколько существующих { -packfile(case: "accusative", number: "many", capitalization: "lower") }, объединяя их в один.
tt_packfile_save_packfile = Сохранить изменения, внесённые в текущий { -packfile(capitalization: "lower") }, на диск.
tt_packfile_save_packfile_as = Сохранить открытый { -packfile(capitalization: "lower") } как новый { -packfile(capitalization: "lower") }, вместо того, чтобы перезаписать исходный.
tt_packfile_install = Скопировать выбранный { -packfile(capitalization: "lower") } в папку { -Data(capitalization: "lower") } активной игры.
tt_packfile_uninstall = Удалить выбранный { -packfile(capitalization: "lower") } из папки { -Data(capitalization: "lower") } активной игры.
tt_packfile_load_all_ca_packfiles = Попробовать одновременно загрузить все { -packedfile(case: "accusative", number: "other", capitalization: "lower") } из каждого заводского { -packfile(case: "genitive", number: "one", capitalization: "lower") } текущей игры в { -app }, используя метод ленивой загрузки { -packedfile(case: "genitive", number: "other", capitalization: "lower") }. Помните, что при попытке сохранить такой { -packfile(number: "one", capitalization: "lower") } Ваш компьютер может взорваться.
tt_packfile_preferences = Открыть диалоговое окно «Параметры» / «Настройки».
tt_packfile_quit = Выйти из { -app }.

-changes-type = Меняет тип { -packfile(case: "genitive", capitalization: "lower") } на
-never_use_type = Вам не следует использовать этот тип для своих модов

tt_change_packfile_type_boot = { -changes-type } «{ -local-packfile_type_boot }». { -never_use_type }.
tt_change_packfile_type_release = { -changes-type } «{ -local-packfile_type_release }». { -never_use_type }.
tt_change_packfile_type_patch = { -changes-type } «{ -local-packfile_type_patch }». { -never_use_type }.
tt_change_packfile_type_mod = { -changes-type } «{ -local-packfile_type_mod }». Используйте этот тип для модов, которые должны появляться в списке Менеджера модов.
tt_change_packfile_type_movie = { -changes-type } «{ -local-packfile_type_movie }». Используйте этот тип для модов, которые должны быть всегда активны и не появляться в списке Менеджера модов.
tt_change_packfile_type_other = { -changes-type } «{ -local-packfile_type_other(includeOriginalText: "true") }». Этот тип используется для { -packfile(case: "genitive", number: "other", capitalization: "lower") }, которые не поддерживают операцию записи. { -never_use_type }.

-if-checked = Если активно
-packfile-saving-not-supported =
    Сохранение подобных { -packfile(case: "genitive", number: "other", capitalization: "lower") } НЕ ПОДДЕРЖИВАЕТСЯ.

tt_change_packfile_type_data_is_encrypted = { -if-checked }, данные { -packedfile(case: "genitive", number: "other", capitalization: "lower") } в этом { -packfile(case: "prepositional", capitalization: "lower") } зашифрованы. { -packfile-saving-not-supported }.
tt_change_packfile_type_index_includes_timestamp = { -if-checked }, индекс { -packedfile(case: "genitive", number: "other", capitalization: "lower") } в этом { -packfile(case: "prepositional", capitalization: "lower") } включает в себя временну́ю метку последнего изменения каждого { -packedfile(case: "genitive", capitalization: "lower") }. Учтите, что подобные { -packfile(number: "other", capitalization: "lower") } НЕ ПОЯВЯТСЯ в списке модов официального загрузчика.
tt_change_packfile_type_index_is_encrypted = { -if-checked }, индекс { -packedfile(case: "genitive", number: "other", capitalization: "lower") } этого { -packfile(case: "genitive", capitalization: "lower") } зашифрован. { -packfile-saving-not-supported }.
tt_change_packfile_type_header_is_extended = { -if-checked }, заголовок этого { -packfile(case: "genitive", capitalization: "lower") } расширен на 20 байт. Наблюдается исключительно в зашифрованных { -packfile(case: "prepositional", number: "other", capitalization: "lower") } игры Arena. { -packfile-saving-not-supported }.
tt_change_packfile_type_data_is_compressed = { -if-checked }, данные каждого { -packedfile(case: "genitive", capitalization: "lower") } в открытом { -packfile(case: "prepositional", capitalization: "lower") } будут сжаты во время сохранения. Если Вы хотите разжать этот { -packfile(capitalization: "lower") }, отключите опцию, а затем сохраните его.

## MyMod menu tips

tt_mymod_new = Открывает диалоговое окно для создания
    нов{ -mymod.gender ->
       *[masculine] ого
        [feminine] ой
    } { -mymod(case: "genitive") }.
tt_mymod_delete_selected = Удаляет
    текущ{ -mymod.gender ->
       *[masculine] ий
        [feminine] ую
        [other] ее
    } { -mymod }.

tt_mymod_import = Перемещает всё содержимое папки { -mymod(case: "genitive") }
    в соответствующий { -packfile(capitalization: "lower") }.
    Если какие-либо файлы были удалены в папке { -mymod(case: "genitive") },
    они также будут удалены и в { -packfile(case: "prepositional",capitalization: "lower") }.
tt_mymod_export = Перемещает всё содержимое { -packfile(case: "genitive",capitalization: "lower") }
    в папку { -mymod(case: "genitive") }.
    Если какие-либо файлы были удалены из соответствующего { -packfile(case: "genitive", capitalization: "lower") },
    они также будут удалены и в папке { -mymod(case: "genitive") }.

## GameSelected menu tips

-tries_to = Будет предпринята попытка

tt_game_selected_launch_game = { -tries_to } запустить { -game_selected(case: "accusative", capitalization: "lower") } на платформе Steam.
tt_game_selected_open_game_data_folder = { -tries_to } открыть папку { -Data(case: "genitive") } { -game_selected(case: "genitive", capitalization: "lower") }, если она существует, в файловом менеджере по умолчанию.
tt_game_selected_open_game_assembly_kit_folder = { -tries_to } открыть папку { -AssemblyKit(case: "genitive") } { -game_selected(case: "genitive", capitalization: "lower") }, если она существует, в файловом менеджере по умолчанию.
tt_game_selected_open_config_folder = { -tries_to } открыть папку с настройками { -app(case: "genitive" ) }, где хранятся файлы конфигурации, { -schema(number: "other", capitalization: "lower") }, а также отчёты об аварийных завершениях работы.

-tt_game_selected =
    Устанавливает «Total War: {$game}» в качестве { -game_selected(case: "genitive", capitalization: "lower") }

tt_game_selected_troy = { -tt_game_selected(game: "Troy") }.
tt_game_selected_three_kingdoms = { -tt_game_selected(game: "Three Kingdoms") }.
tt_game_selected_warhammer_2 = { -tt_game_selected(game: "Warhammer 2") }.
tt_game_selected_warhammer = { -tt_game_selected(game: "Warhammer") }.
tt_game_selected_thrones_of_britannia = { -tt_game_selected(game: "Thrones of Britannia") }.
tt_game_selected_attila = { -tt_game_selected(game: "Attila") }.
tt_game_selected_rome_2 = { -tt_game_selected(game: "Rome 2") }.
tt_game_selected_shogun_2 = { -tt_game_selected(game: "Shogun 2") }.
tt_game_selected_napoleon = { -tt_game_selected(game: "Napoleon") }.
tt_game_selected_empire = { -tt_game_selected(game: "Empire") }.
tt_game_selected_arena = { -tt_game_selected(game: "Arena") }.

## Special Stuff menu tips

tt_optimize_packfile = Проверить и удалить любые записи в таблицах
    { -db(case: "genitive", number: "other", capitalization: "lower") } и
    { -loc(case: "prepositional", number: "other", capitalization: "lower") }, идентичные заводским.
    Это означает, что Ваш мод будет содержать только изменённые записи, избегая несовместимостей с другимимодами.
tt_patch_siege_ai = Применить патч и очистить экспортируемую карту
    { -packfile(case: "genitive", capitalization: "lower") }.
    Это может исправить искусственный интеллект осады (Siege AI),
    если таковой представлен, а также удалить ненужные XML-файлы, занимающие место в
    { -packfile(case: "prepositional", capitalization: "lower") },
    таким образом, уменьшая
    { -packfile.gender ->
       *[masculine] его
        [feminine] её
    } размер.

## About menu tips

tt_about_about_qt =
    Информация о Qt, наборе инструментов пользовательского интерфейса, который использовался для создания этой программы.
tt_about_about_rpfm =
    Информация { -app.startsWith ->
       *[consonant] О
        [vowel] Об
    } { -app(case: "prepositional") }.
tt_about_open_manual = Открыть руководство по использованию { -app(case: "genitive") } в программе для просмотра PDF-файлов.
tt_about_patreon_link = Открыть страницу { -app(case: "genitive") } на Patreon.
    Пожалуйста, ознакомьтесь с ней даже если Вы не заинтересованы в том, чтобы стать моим патроном —
    я время от времени публикую там новости о грядущих обновлениях и новых возможностях, находящихся в разработке.
tt_about_check_updates = Проверяет, есть ли доступные обновления для { -app(case: "genitive") }.
tt_about_check_schema_updates = Проверяет, есть ли доступные обновления для
    { -schema(case: "genitive", number: "other", capitalization: "lower") }.
    Это следует делать после каждого обновления игры.

## global_search_ui/mod.rs

global_search = Глобальный поиск
global_search_info = Что ищем?
global_search_search = Найти
global_search_replace = Заменить
global_search_replace_all = Заменить всё
global_search_clear = Очистить
global_search_case_sensitive = Учёт регистра
global_search_use_regex = Рег. выражение
# TODO: fuzzy, needs context
global_search_search_on = Поиск по

global_search_all = Всё
global_search_db = { -db(number: "other") }
global_search_loc = { -loc(number: "other") }
global_search_txt = Текст
global_search_schemas = { -schema(number: "other") }

## Filter Dialogues

-matches = Совпадения

global_search_db_matches = { -matches } { -db(case: "genitive", number: "other", capitalization: "lower") }
global_search_loc_matches = { -matches } { -loc(case: "genitive", number: "other", capitalization: "lower") }
global_search_txt_matches = { -matches } текстовых файлов
global_search_schema_matches = { -matches } { -schema(case: "genitive", number: "other", capitalization: "lower") }

global_search_match_packedfile_column = { -packedfile }/Столбец
global_search_match_packedfile_text = { -packedfile }/Текст

# TODO: fuzzy, needs context
global_search_versioned_file = Версийный файл (тип, имя)/Имя столбца
global_search_definition_version = Версия определения
global_search_column_index = Индекс столбца

## tips

-include-on-search = Искать по

tt_global_search_use_regex_checkbox = Включить поиск с использованием регулярного выражения. В случае, если регулярное выражение некорректно, { -app } интерпретирует его как обычный поисковый запрос.
tt_global_search_case_sensitive_checkbox = Включить чувствительный к регистру поиск. В таком режиме учитываются ВЕРХНИЙ/нижний регистры символов.
tt_global_search_search_on_all_checkbox = { -include-on-search } { -packedfile(case: "dative", number: "other", capitalization: "lower") } / { -schema(case: "dative", number: "other", capitalization: "lower") }.
tt_global_search_search_on_dbs_checkbox = { -include-on-search } { -db(case: "dative", number: "other", capitalization: "lower", includeOriginalText: "true") }.
tt_global_search_search_on_locs_checkbox = { -include-on-search } { -loc(case: "dative", number: "other", capitalization: "lower", includeOriginalText: "true") }.
tt_global_search_search_on_texts_checkbox = { -include-on-search } текстовым { -packedfile(case: "dative", number: "other", capitalization: "lower") }.
tt_global_search_search_on_schemas_checkbox = Включить для поиска загруженную в данный момент { -schema(number: "genitive", capitalization: "lower") }.

## Open PackedFile Dialog

open_packedfile_dialog_1 = Вы уверены?
open_packedfile_dialog_2 = Один или более { -packedfile(number: "many", capitalization: "lower" ) }, которые Вы пытаетесь перезаписать или удалить, открыты в настоящий момент. Вы уверены, что хотите продолжить? Вариант «Да» закроет эти { -packedfile(number: "other", capitalization: "lower" ) }.

## TreeView Text/Filter

treeview_aai = AaI
treeview_autoexpand = Раскрывать совпадения
treeview_expand_all = &Раскрыть Все
treeview_collapse_all = &Свернуть все

## TreeView Tips

-tt-files-overwrite =
    Существующие файлы { $overwritten ->
       *[true] { "" }
        [false] не
    } будут перезаписаны!
-tt-open-dialog = Открывает диалоговое окно создания

tt_context_menu_add_file = Добавить один или более файлов в рабочий { -packfile(capitalization: "lower") }.
    { -tt-files-overwrite(overwritten: "false") }
tt_context_menu_add_folder = Добавить папку в рабочий { -packfile(capitalization: "lower") }.
    { -tt-files-overwrite(overwritten: "false") }
tt_context_menu_add_from_packfile = Добавить файлы из другого
    { -packfile(case: "genitive", capitalization: "lower") } в рабочий { -packfile(capitalization: "lower") }.
    { -tt-files-overwrite(overwritten: "false") }
tt_context_menu_check_tables = Проверить все { -db(number: "other", capitalization: "lower", includeOriginalText: "true") }
    рабочего { -packfile(case: "genitive", capitalization: "lower") } на предмет наличия ошибок зависимостей.
tt_context_menu_new_folder = { -tt-open-dialog }
    пустой папки.
    Из-за природы самих { -packfile(case: "genitive", number: "other", capitalization: "lower") },
    если такая папка будет оставаться пустой на момент сохранения { -packfile(case: "genitive", capitalization: "lower") },
    она НЕ БУДЕТ СОХРАНЕНА.
tt_context_menu_new_packed_file_anim_pack = { -tt-open-dialog }
    { -animpack(case: "genitive") }.
tt_context_menu_new_packed_file_db = { -tt-open-dialog }
    { -db(case:"genitive", capitalization: "lower", includeOriginalText: "true") },
    которые используются для изменения большинства аспектов игры.
tt_context_menu_new_packed_file_loc = { -tt-open-dialog }
    { -loc(case:"genitive", capitalization: "lower", includeOriginalText: "true") },
    которые используются для хранения большинства текстовых строк, выводимых в игре.
tt_context_menu_new_packed_file_text = { -tt-open-dialog }
    произвольного текстового файла.
    Расширение такого файла может быть любым: .xml, .lua, .txt, и т. д.
tt_context_menu_new_queek_packed_file = { -tt-open-dialog }
    { -packedfile(case: "genitive", capitalization: "lower") } в зависимости от контекста.
    Например, если Вы запустите это действие, находясь в папке «/text»,
    это создаст { -packedfile(capitalization: "lower") } типа «{ -loc(includeOriginalText: "true") }».
tt_context_menu_mass_import_tsv = Импортирует массив TSV-файлов, автоматически проверяя,
    являются ли они
    { -db(case: "instrumental", number: "other", capitalization: "lower", includeOriginalText: "true") },
    { -loc(case: "instrumental", number: "other", capitalization: "lower", includeOriginalText: "true") },
    или же некорректными файлами TSV,
    и помещает их как { -packedfile(number: "other", capitalization: "lower") } правильного типа.
    { -tt-files-overwrite }
tt_context_menu_mass_export_tsv = Экспортирует каждый
    { -packedfile(capitalization: "lower") }
    типа «{ -db(capitalization: "lower", includeOriginalText: "true") }» или
    «{ -loc(capitalization: "lower", includeOriginalText: "true") }»
    из этого { -packfile(case: "genitive", capitalization: "lower") } как TSV-файлы.
    { -tt-files-overwrite }
tt_context_menu_merge_tables =
    Объединяет несколько { -db(number: "many", capitalization: "lower", includeOriginalText: "true") }
    или { -loc(number: "many", capitalization: "lower", includeOriginalText: "true") }
    в один { -packedfile(capitalization: "lower") }.
tt_context_menu_update_tables =
    Обновляет { -db(case: "accusative", capitalization: "lower") } до последней известной рабочей версии
    { -db.gender ->
       *[masculine] его
        [feminine] её
    } текущ{ -game_selected.gender ->
       *[masculine] его
        [feminine] ей
    }
    { -schema(case: "genitive", capitalization: "lower") }
    { -game_selected(case: "genitive", capitalization: "lower") }.
tt_context_menu_delete = Удалить выбранный файл или папку.

tt_context_menu_extract = Извлечь выбранный файл или папку из этого { -packfile(case:"genitive") }.
tt_context_menu_rename = Переименовать выбранный файл или папку.
    Учтите, что пробелы и другие пробельные символы в именах НЕ РАЗРЕШЕНЫ,
    а дублирующиеся имена в той же папке не будут переименованы.
tt_context_menu_open_decoder = Открывает текущую
    { -db(case: "accusative", capitalization: "lower", includeOriginalText: "true") }
    в Декодере { -db(case: "genitive") }.
    Используется для создания или обновления
    { -schema(case: "accusative", number: "other", capitalization: "lower") }.
tt_context_menu_open_dependency_manager = Открывает список
    { -packfile(number: "many", capitalization: "lower") },
    на которые ссылается открытый { -packfile(capitalization: "lower") }.
tt_context_menu_open_containing_folder = Открывает местонахождение текущего
    { -packfile(case: "genitive", capitalization: "lower") }
    в файловом менеджере по умолчанию.
tt_context_menu_open_with_external_program = Открывает { -packedfile(capitalization: "lower") } во внешнем редакторе.
tt_context_menu_open_notes = Открывает заметки рабочего
    { -packfile(case: "genitive", capitalization: "lower") }
    во вторичном обзоре, не закрывая текущий
    { -packedfile(capitalization: "lower") }, открытый в главном обзоре.

tt_filter_autoexpand_matches_button = Автоматически раскрывает совпадения поиска.
    ВНИМАНИЕ: фильтрация результатов поиска в больших
    { -packfile(case: "prepositional", number: "other", capitalization: "lower") }
    (более чем с 10-ю тыс. { -packedfile(number: "many", capitalization: "lower") }, таких, как data.pack)
    могут серьёзно замедлить { -app(case: "accusative") }.
    Вас предупредили.
tt_filter_case_sensitive_button = Включить/выключить чувствительную к регистру символов фильтрацию
    дерева { -packedfile(case: "genitive", number: "other", capitalization: "lower") }.

# TODO: fuzzy, needs context
packedfile_editable_sequence = Редактируемая последовательность

## Rename Dialogues

rename_selection = Переименовать выделенное
rename_selection_instructions = Инструкция
rename_selection_placeholder = Новое название выбранных { -packedfile(case: "genitive", number: "other", capitalization: "lower") }. Переменная {"{"}x{"}"} представляет текущее имя файла.

## Mass-Import

mass_import_tsv = Массовый импорт TSV-файлов
mass_import_num_to_import = Файлов к импорту: 0.
mass_import_use_original_filename = Использовать исходное имя файла:
mass_import_import = Импортировать
mass_import_default_name = new_imported_file

mass_import_select = Выберите файлы TSV для импорта…

files_to_import = Импортируемые файлы: {"{"}{"}"}.

## Table

decoder_title = Декодер { -packedfile(case: "genitive", number: "many") }
table_dependency_manager_title = Менеджер зависимостей
table_filter_case_sensitive = Регистрозависимый
table_enable_lookups = Используйте поиск

## Contextual Menu for TreeView

context_menu_add = Доб&авить…
context_menu_create = &Создать…
context_menu_open = &Открыть…

context_menu_add_file = Доб&авить файл
context_menu_add_files = Добавить файлы
context_menu_add_folder = Добавить &папку
context_menu_add_folders = Добавить папки
context_menu_add_from_packfile = Добавить из { -packfile(case: "genitive") }
context_menu_select_packfile = Выбрать { -packfile }
context_menu_extract_packfile = Извлечь { -packfile }

-local-create = Создать

context_menu_new_folder = { -local-create } папку
context_menu_new_packed_file_anim_pack = { -local-create } { -animpack }
context_menu_new_packed_file_db = { -local-create } { -db(case: "accusative", includeOriginalText: "true") }
context_menu_new_packed_file_loc = { -local-create } { -loc(case: "accusative", includeOriginalText: "true") }
context_menu_new_packed_file_text = { -local-create } Текстовый файл
context_menu_new_queek_packed_file = Новый Queek-файл

context_menu_mass_import_tsv = Массовый импорт TSV
context_menu_mass_export_tsv = Массовый экспорт TSV
context_menu_mass_export_tsv_folder = Выберите папку назначения
context_menu_rename = Пе&реименовать
context_menu_delete = У&далить
context_menu_extract = Извл&ечь

context_menu_open_decoder = &Открыть в Декодере
context_menu_open_dependency_manager = Открыть Менеджер &зависимостей
context_menu_open_containing_folder = Открыть &содержащую папку
context_menu_open_with_external_program = Открыть во внешнем &редакторе
context_menu_open_notes = Открыть за&метки

context_menu_check_tables = &Проверить { -db(case: "accusative", number: "other") }
context_menu_merge_tables = &Объединить { -db(case: "accusative", number: "other") }
context_menu_update_table = Об&новить { -db(case: "accusative", number: "other") }

## Shortcuts

menu_bar_packfile_section = Меню «{ -packfile }»
menu_bar_mymod_section = Меню «{ -mymod }»
menu_bar_view_section = Меню «Вид»
menu_bar_game_selected_section = Меню «{ -game_selected }»
menu_bar_about_section = Меню «{ -app.startsWith ->
       *[consonant] О
        [vowel] Об
    } { -app }»
packfile_contents_tree_view_section = Контекстное меню содержимого { -packfile(case: "genitive", capitalization: "lower") }
packed_file_table_section = Контекстное меню
    { -packedfile(case: "genitive", capitalization: "lower") }-{ -db(case: "genitive", capitalization: "lower") }
packed_file_decoder_section = Декодер { -packedfile(case: "genitive", number: "other", capitalization: "lower") }

shortcut_esc = Esc
shortcut_csp = Ctrl+Shift+P

shortcut_title = Быстрые клавиши
shortcut_text = Сочетание клавиш
shortcut_section_action = Раздел/Команда

## Settings

settings_title = Параметры

settings_game_paths_title = Расположение игровых файлов
settings_extra_paths_title = Прочие расположения
settings_paths_mymod = Папка { -mymod(case: "genitive") }
settings_paths_mymod_ph = Это папка, в которой будут храниться все относящиеся к функции "{ -mymod }" файлы.

settings_paths_zip = Путь до .exe-файла 7Zip
settings_paths_zip_ph = Полный путь до исполняемого файла 7Zip.

settings_game_label = Папка игры
settings_asskit_label = Папка { -AssemblyKit(case:"genitive") }
settings_game_line_ph = Это папка, где находится .exe-файл установленной игры {"{"}{"}"}.
settings_asskit_line_ph = Это папка, где установлен{ -AssemblyKit.gender ->
       *[masculine] { "" }
        [feminine] а
        [other] о
    } { -AssemblyKit } для {"{"}{"}"}.

settings_ui_title = Настройки интерфейса
settings_table_title = Настройки { -db(case:"genitive", number: "other", capitalization: "lower") }

settings_ui_language = Язык (требует перезапуска):
settings_ui_dark_theme = Включить тёмную тему:
settings_ui_table_adjust_columns_to_content = Подогнать столбцы под содержимое:
# TODO: fuzzy, needs context
settings_ui_table_disable_combos = Отключить списки быстрого выбора в { -db(case: "prepositional", number: "many", capitalization: "lower") }:
settings_ui_table_extend_last_column_label = Расширять последний столбец в { -db(case: "prepositional", number: "many", capitalization: "lower") }:
settings_ui_table_tight_table_mode_label = Включить компактный режим в { -db(case: "prepositional", number: "many", capitalization: "lower") }:
settings_ui_table_remember_column_visual_order_label = Запоминать порядок столбцов (только вид):
settings_ui_table_remember_table_state_permanently_label = Запоминать состояние { -db(case: "genitive", number: "many", capitalization: "lower") } между { -packfile(case: "prepositional", number: "many", capitalization: "lower") }:
settings_ui_window_start_maximized_label = Запускать во весь экран:
settings_ui_window_hide_background_icon = Спрятать иконку игры на заднем плане:

settings_select_file = Выбрать файл
settings_select_folder = Выбрать папку

settings_extra_title = Дополнительные настройки
settings_default_game = Игра по умолчанию:
settings_check_updates_on_start = Проверять обновления при запуске:
settings_check_schema_updates_on_start = Проверять обновления { -schema(case: "genitive", number: "many", capitalization: "lower") } при запуске:
settings_check_template_updates_on_start = Проверять обновления шаблонов при запуске:
settings_allow_editing_of_ca_packfiles = Позволить редактирование { -packfile(case: "genitive", number: "many", capitalization: "lower") } { -CA(case:"genitive") }:
settings_optimize_not_renamed_packedfiles = Оптимизировать не переименованные { -packfile(number: "other", capitalization: "lower") }:
settings_use_lazy_loading = Использовать метод ленивой загрузки для { -packfile(case: "genitive", number: "many", capitalization: "lower") }:
settings_disable_uuid_regeneration_tables = Отключить повторную генерацию UUID для { -db(case: "genitive", number: "many", capitalization: "lower", includeOriginalText: "true") }:
settings_packfile_treeview_resize_to_fit = Подгонять размер дерева { -packedfile(case: "genitive", number: "many", capitalization: "lower") } под ширину содержимого:
settings_table_resize_on_edit = Подгонять размер { -db(case: "genitive", number: "many", capitalization: "lower") } под ширину содержимого во время правок:

settings_debug_title = Настройки отладки
settings_debug_missing_table = Проверять отсутствующие определения { -db(case: "genitive", number: "many", capitalization: "lower") }
settings_debug_enable_debug_menu = Включить меню отладки

settings_diagnostics_title = Настройки диагностики
settings_diagnostics_show_panel_on_boot = Включить инструмент диагностики:
settings_diagnostics_trigger_on_open = Запускать диагностику при открытии { -packfile(case: "genitive", capitalization: "lower") }:
settings_diagnostics_trigger_on_edit = Запускать диагностику при редактировании { -db(case: "genitive", number: "many", capitalization: "lower") }:

settings_text_title = Настройки текстового редактора

settings_warning_message =
    <p>
        <b style="color:red;">
        ВНИМАНИЕ: Изменение большинства этих параметров потребует перезапуска
        { -app(case: "genitive") } для применения!
        </b>
    </p>
    <p></p>

## Settings Tips

-if_enabled = При включении

# Preserve The Black Speech
tt_ui_global_use_dark_theme_tip = <i>Ash nazg durbatulûk, ash nazg gimbatul, ash nazg thrakatulûk, agh burzum-ishi krimpatul</i>
tt_ui_table_adjust_columns_to_content_tip = { -if_enabled }, ширина всех столбцов открываемых { -db(case: "genitive", number: "many", capitalization: "lower", includeOriginalText: "true") } или { -loc(case: "genitive", number: "many", capitalization: "lower", includeOriginalText: "true") } будет автоматически подогнана под их содержимое.
    В противном случае, столбцы будут иметь предопределенный размер. Так или иначе, Вы сможете изменить их ширину вручную после открытия.
    ВНИМАНИЕ: { -if_enabled }, открытие очень больших { -db(case: "genitive", number: "many", capitalization: "lower") } может занять больше времени.
tt_ui_table_disable_combos_tip = При отключении, списки быстрого выбора подходящих значений более не будут отображаться при редактировании { -db(case: "genitive", number: "many", capitalization: "lower") }. Это означает, что ни списки быстрого выбора, ни функция автозаполнения не будут работать во время изменения значений в { -db(case: "prepositional", number: "many", capitalization: "lower") }.
    Теперь-то ты доволен, Baldy?
tt_ui_table_extend_last_column_tip = { -if_enabled }, последний столбец { -db(case: "genitive", number: "many", capitalization: "lower", includeOriginalText: "true") } и { -loc(case: "genitive", number: "many", capitalization: "lower", includeOriginalText: "true") } будет растягиваться до достижения правого края по мере необходимости.
tt_ui_table_tight_table_mode_tip = { -if_enabled }, используемое вертикальное пространство в { -db(case: "prepositional", number: "many", capitalization: "lower") } будет сокращено до минимума, позволяя видеть больше строк одновременно.
tt_ui_table_remember_column_visual_order_tip = { -if_enabled }, { -app } будет запоминать визуальный порядок столбцов { -db(case: "genitive", number: "many", capitalization: "lower") } и { -loc(case: "genitive", number: "many", capitalization: "lower") } после их закрытия.
tt_ui_table_remember_table_state_permanently_tip = { -if_enabled }, { -app } будет запоминать полное состояние { -packedfile(case: "genitive", number: "many", capitalization: "lower") } { -db(case: "genitive", number: "many", capitalization: "lower", includeOriginalText: "true") } и { -loc(case: "genitive", number: "many", capitalization: "lower", includeOriginalText: "true") }, включая результаты фильтрации, порядок столбцов и столбец, используемый для сортировки { -db(case: "genitive", capitalization: "lower") }, даже если Вы закроете { -app } и откроете снова. Оставьте отключённым, если не хотите такого поведения встроенного редактора.
tt_ui_window_start_maximized_tip = { -if_enabled }, { -app } будет запускаться развёрнутым во весь экран.


tt_extra_network_check_updates_on_start_tip = { -if_enabled }, { -app } будет проверять наличие обновлений при запуске, а также уведомлять Вас о них.
    Решение о загрузке и применении обновлений остаётся за Вами.
tt_extra_network_check_schema_updates_on_start_tip = { -if_enabled }, { -app } будет проверять наличие обновлений { -schema(case: "genitive", number: "many", capitalization: "lower") } при запуске, а также давать Вам возможность тут же их обновить.
tt_extra_packfile_allow_editing_of_ca_packfiles_tip = По умолчанию, лишь { -packfile(number: "other", capitalization: "lower") } типов «{ -local-packfile_type_mod }» и «{ -local-packfile_type_movie }» разрешены для редактирования, т. к. это единственные типы { -packfile(case: "genitive", number: "other", capitalization: "lower") }, используемые в модах.
    Если Вы включите эту опцию, Вы также сможете редактировать { -packfile(case: "genitive", number: "other", capitalization: "lower") } типов «{ -local-packfile_type_boot }», «{ -local-packfile_type_release }» и «{ -local-packfile_type_patch }».
    ВНИМАНИЕ: не перезаписывайте оригинальные { -packfile(case: "accusative", number: "other", capitalization: "lower") }!
tt_extra_packfile_optimize_not_renamed_packedfiles_tip = { -if_enabled }, { -app } будет оптимизировать { -db(case: "accusative", number: "other", capitalization: "lower", includeOriginalText: "true") } и { -loc(case: "accusative", number: "other", capitalization: "lower", includeOriginalText: "true") } с теми же именами, что и их оригинальные прообразы, во время запуска процедуры «{ special_stuff_optimize_packfile }».
    Обычно, такие файлы предназначены для переопределения заводских записей, так что по умолчанию они игнорируются этой функцией. Тем не менее, иногда возникает необходимость оптимизировать и их (например, когда { -AssemblyKit } включает в себя слишком много файлов) — вот почему существует эта опция.
tt_extra_packfile_use_lazy_loading_tip = { -if_enabled }, данные в { -packfile(case: "prepositional", number: "other", capitalization: "lower") } будут подгружаться с диска по мере необходимости, вместо того, чтобы быть загруженными в оперативную память целиком.
    Это существенно уменьшает занимаемую оперативную память, но в том случае, если открытый { -packfile(capitalization: "lower") } будет также перезаписан другой программой, Вы потеряете свой прогресс, а Ваш { -packfile(capitalization: "lower") } будет не восстановить.
    Если Ваш процесс создания модов протекает преимущественно в папке { -Data } игры «Warhammer 2», ОСТАВЬТЕ ЭТУ ОПЦИЮ ВЫКЛЮЧЕННОЙ, так как ошибка в { -AssemblyKit } может вызвать повреждение или непреднамеренное удаление { -packfile(case: "genitive", number: "many", capitalization: "lower") }, пока эта опция остаётся включённой.
tt_extra_disable_uuid_regeneration_on_db_tables_label_tip = Включите эту опцию, если Вы планируете помещать экспортируемые { -db(case: "accusative", number: "other", capitalization: "lower", includeOriginalText: "true" ) } в двоичном формате в репозиторий с функцией контроля версий, такой как Git, SVN, или любой другой.

tt_debug_check_for_missing_table_definitions_tip = { -if_enabled }, { -app } попытается раскодировать КАЖД{ -db.gender ->
        [masculine] ОГО
        [feminine] УЮ
       *[other] ОЕ
    } { -db(case: "accusative", capitalization: "upper") } в рабочем { -packfile(case: "prepositional", capitalization: "lower") } при открытии или смене { -game_selected(case: "genitive", capitalization: "lower") }, и выводит все { -db(case: "accusative", number: "other", capitalization: "lower") } без { -schema(case: "genitive", capitalization: "lower") } в файл \"missing_table_definitions.txt\".
    ЭТА ФУНКЦИЯ ПРЕДНАЗНАЧЕНА ИСКЛЮЧИТЕЛЬНО ДЛЯ ОТЛАДКИ И СИЛЬНО ЗАМЕДЛЯЕТ РАБОТУ, НЕ ВКЛЮЧАЙТЕ БЕЗ НЕОБХОДИМОСТИ.

tt_diagnostics_enable_diagnostics_tool_tip = { -if_enabled }, панель диагностики будет появляться при запуске.
tt_diagnostics_trigger_diagnostics_on_open_tip = { -if_enabled },
    запускает полную диагностику { -packfile(case:"genitive", capitalization: "lower") }
    во время { -packfile.gender ->
       *[masculine] его
        [feminine] её
    } открытия.
tt_diagnostics_trigger_diagnostics_on_table_edit_tip = { -if_enabled }, запускает ограниченную диагностику при каждом изменении { -db(case: "genitive", capitalization: "lower") }.

# CA_VP8 Videos

format = Формат:
version = Версия:
header_len = Длина заголовка:
codec_four_cc = FourCC кодек:
width = Ширина:
height = Высота:
ms_per_frame = Мс. на кадр:
num_frames = Кол-во кадров:
largest_frame = Наибольший кадр:
mystery_number = Загадочное число:
offset_frame_table = Сдвиг таблицы кадра:
framerate = Частота кадров:
timebase = Развёртка:
x2 = Загадочная величина:

convert_to_camv = Конвертировать в CAMV
convert_to_ivf = Конвертировать в IVF

notes = Заметки

external_current_path = Текущее расположение редактируемого файла:
stop_watching = Прекратить отслеживать изменения
open_folder = Открыть папку в файловом менеджере

game_selected_changed_on_opening =
    { -game_selected } измен{ -game_selected.gender ->
       *[masculine] ён
        [feminine] ена
    } на {"{"}{"}"}, так как открыт{ -packfile.gender ->
        [masculine] ый
        [feminine] ая
       *[other] ое
    } { -packfile(capitalization: "lower") } несовместим с
    { -game_selected(case: "instrumental", capitalization: "lower") }.

## Extra stuff I don't remember where it goes.

rpfm_title = { -app(full: "true") }
delete_mymod_0 =
    <p>
        Вы собираетесь удалить { -mymod.gender ->
            [masculine] этот
            [feminine] эту
           *[other] это
        } <i>'{ -mymod }'</i>
        с Вашего диска.
    </p>
    <p>
        Вы не сможете { -mymod.gender ->
           *[masculine] его
            [feminine] её
        } восстановить.
    </p>
    <p>Вы уверены?</p>
delete_mymod_1 = <p>Есть несохранённые изменения.</p><p>Вы уверены?</p>

-update_caution =
    Убедитесь, что все изменения сохранены, прежде, чем нажать «{ update_button }», иначе они могут быть потеряны.
-good_luck = Удачи в следующий раз! :)
-check_error = Возникла ошибка при проверке обновлений

api_response_success_new_stable_update =
    <h4>Обнаружена новая стабильная версия: {"{"}{"}"}</h4>
    <p>{ -update_caution }</p>
api_response_success_new_beta_update =
    <h4>Обнаружена новая бета-версия: {"{"}{"}"}</h4>
    <p>{ -update_caution }</p>
api_response_success_new_update_hotfix =
    <h4>Обнаружен новый патч/исправление: {"{"}{"}"}</h4>
    <p>{ -update_caution }</p>
api_response_success_no_update =
    <h4>Обновлений не найдено</h4>
    <p>{ -good_luck }</p>
api_response_success_unknown_version =
    <h4>{ -check_error }</h4>
    <p>
        Возникла ошибка при получении номера последней версии либо
        номера текущей версии. Это может значить, что я накосячил
        с названием последней версии.
        Если Вы это видите, пожалуйста, сообщите об этом здесь:
    </p>
    <p>
        <a href=\"https://github.com/Frodo45127/rpfm/issues\">
            https://github.com/Frodo45127/rpfm/issues
        </a>
    </p>
api_response_error = <h4>{ -check_error } :(</h4> {"{"}{"}"}

schema_no_update =
    <h4>Обновлений { -schema(case: "genitive", capitalization: "lower") } не найдено</h4>
    <p>{ -good_luck }</p>
schema_new_update =
    <h4>
    Доступны обновления { -schema(case: "genitive", capitalization: "lower") }
    </h4>
    <p>Обновить { -schema(case: "accusative", number: "many", capitalization: "lower") }?</p>

template_no_update =
    <h4>Обновлений шаблонов не найдено</h4>
    <p>{ -good_luck }</p>
template_new_update =
    <h4>Доступны обновления шаблонов</h4>
    <p>Обновить шаблоны?</p>

api_response_schema_error =
    <h4>{ -check_error } :(</h4>
    <p>
        Возникла ошибка при подключении к github.com.
        Убедитесь, что Вы можете открыть
        <a href=\"https://api.github.com\">https://api.github.com</a>, и попытайтесь снова.
    </p>
schema_update_success =
    <h4>{ -schema(number: "other") } обновлены и перезагружены</h4>
    <p>Вы можете продолжить работу в { -app(case: "prepositional") }.</p>
template_update_success =
    <h4>Шаблоны обновлены и перезагружены</h4>
    <p>Вы можете продолжить работу в { -app(case: "prepositional") }.</p>

files_extracted_success = Файлов извлечено: {"{"}{"}"}\n
    Ошибок не обнаружено.
mymod_delete_success =
    { -mymod } успешно удал{ -mymod.gender ->
        [masculine] ён
        [feminine] ена
       *[other] ено
    }: «{"{"}{"}"}»

game_selected_unsupported_operation = Операция недоступна для
    { -game_selected(case: "genitive", capitalization: "lower") }.

optimize_packfile_success =
    { -packfile } успешно оптимизирован{ -packfile.gender ->
       *[masculine] { "" }
        [feminine] а
        [other] о
    }.
update_current_schema_from_asskit_success =
    { -schema.gender ->
       *[masculine] Текущий загруженный { -schema(capitalization: "lower") } успешно обновлён.
        [feminine] Текущая загруженная { -schema(capitalization: "lower") } успешно обновлена.
        [other] Текущее загруженное { -schema(capitalization: "lower") } успешно обновлено.
    }
generate_schema_diff_success = Diff успешно сгенерирован.
settings_font_title = Настройки шрифтов

title_success = Успех!
title_error = Ошибка!

rename_instructions = <p>Всё просто, хоть и не очень понятно без примера, так что следите за руками:</p>
    <ul>
        <li>Допустим, Вы переименовываете две папки или файла с именами «любит» и «не любит».</li>
        <li>Напишите «{ -app } {"{"}x{"}"} меня» в поле ниже.</li>
        <li>Нажмите «{ rewrite_selection_accept }».</li>
        <li>{ -app } назовёт Ваши файлы «{ -app } любит меня» и «{ -app } не любит меня» соответственно.</li>
    </ul>
    <p>И, раз уж мы здесь, это также работает и с числами. Если получаемый текст — это число, разумеется.</p>

update_table_success = { -db } обновл{ -db.gender ->
        [masculine] ён
        [feminine] ена
       *[other] ено
    } с версии '{"{"}{"}"}' до версии '{"{"}{"}"}'.
no_errors_detected = Ошибок не обнаружено.
original_data = Исходные данные: '{"{"}{"}"}'
column_tooltip_1 = Этот столбец ссылается на:
column_tooltip_2 = И ещё больше. А именно: {"{"}{"}"}. Слишком много, чтобы отобразить здесь.
column_tooltip_3 = Поля, которые ссылаются на этот столбец:
column_tooltip_4 = Это поле ожидает путь к файлу в качестве значения.
column_tooltip_5 = Это поле ожидает название файла, расположенному по этому пути:

tsv_select_title = Выбрать TSV-файл для импорта…
tsv_export_title = Экспортировать TSV-файл…

rewrite_selection_title = Перезаписать выделенное
rewrite_selection_instructions_title = Инструкция
rewrite_selection_instructions =
    <p>Легенда гласит:</p>
    <ul>
    <li>{"{"}x{"}"} представляет текущее значение.</li>
    <li>{"{"}y{"}"} представляет текущий столбец.</li>
    <li>{"{"}z{"}"} представляет текущую строку.</li>
    </ul>
rewrite_selection_is_math = Арифметическая операция?
rewrite_selection_placeholder = Введите новое имя
rewrite_selection_accept = Принять

context_menu_apply_submenu = &Применить…
context_menu_clone_submenu = &Клонировать…
context_menu_copy_submenu = &Копировать…
context_menu_add_rows = Добавить &строку
context_menu_insert_rows = Вс&тавить строку
context_menu_delete_rows = У&далить строку
context_menu_rewrite_selection = &{ rewrite_selection_title }
context_menu_clone_and_insert = К&лонировать и вставить
context_menu_clone_and_append = Клонировать и вставить снизу
context_menu_copy = &Копировать
context_menu_copy_as_lua_table = Копировать как { -db(case: "accusative", capitalization: "lower") } &Lua
context_menu_paste = &Вставить
context_menu_search = &Поиск
context_menu_sidebar = Боковая пане&ль
context_menu_import_tsv = &Импортировать TSV
context_menu_export_tsv = &Экспортировать TSV
context_menu_invert_selection = &Обратить выделение
context_menu_reset_selection = С&бросить выделение
context_menu_resize_columns = Изменить шоирину столбцов
context_menu_undo = От&менить
context_menu_redo = Повто&рить
context_menu_cascade_edition = Переименовать все ссылки

header_column = <b><i>Название столбца</i></b>
header_hidden = <b><i>Скрытый</i></b>
header_frozen = <b><i>Замороженный</i></b>

file_count = Число файлов:
file_paths = Расположения файлов:
animpack_unpack = Распаковать

special_stuff_repack_animtable = Упаковать { -animpack }
tt_repack_animtable = Эта команда переупаковывает { -animpack(case: "accusative", capitalization: "lower") } (если таковые найдены) обратно в формат { -animpack(case: "genitive", capitalization: "lower") }.

load_template = Загрузить шаблон
load_templates_dialog_title = { load_template }
load_templates_dialog_accept = { load_template }

nested_table_title =
    Вложенн{ -db.gender ->
        [masculine] ый
        [feminine] ая
       *[other] ое
    } { -db(capitalization: "lower") }
nested_table_accept = Принять

about_check_template_updates = Проверка обновлений шаблона
uodate_templates_success = Шаблоны успешно обновлены.
# TODO: typo in translation key?
tt_uodate_templates = Это действие пытается обновить Ваши шаблоны.

integer_1 = Неизвестное целочисленное значение 1:
integer_2 = Неизвестное целочисленное значение 2:

settings_update_channel = Канал обновлений
update_success_main_program =
    <h4>
        { -app } успешно обновл{ -app.gender ->
            [masculine] ён
            [feminine] ена
           *[other] ено
        }!
    </h4>
    <p>
        Чтобы узнать, что нового в этой версии, прочитайте заметки к выпуску:
        <a href='file:///{"{"}{"}"}'>
            CHANGELOG.md
        </a>.
        Если Вы обновились до бета-версии, последние изменения находятся в разделе «Unreleased».
    </p>
    <p>
        Перезапустите { -app } для применения изменений.
    </p>

settings_autosave_interval = Интервал автосохранения (в мин.)
autosaving = Автосохранение в процессе…
autosaved = Автоматически сохранено
error_autosave_non_editable =
    { -packfile.gender ->
        [masculine] Этот
        [feminine] Эта
       *[other] Это
    } { -packfile(capitalization: "lower") } не может быть автоматически сохранён.

settings_ui_table_use_old_column_order_label = Использовать прежний порядок столбцов (сначала ключи):

context_menu_paste_as_new_row = Вставить новой строкой

gen_loc_diagnostics = Диагностика
diagnostics_button_check_packfile = Проверить { -packfile(capitalization: "lower") }
diagnostics_button_check_current_packed_file =
    Проверить текущ{ -packedfile.gender ->
        [masculine] ий
        [feminine] ую
       *[other] ее
    } { -packedfile(case: "accusative", capitalization: "lower") }
diagnostics_button_error = Ошибки
diagnostics_button_warning = Предупреждения
diagnostics_button_info = Сообщения
diagnostics_button_only_current_packed_file =
    Только открытые { -packedfile(number: "other", capitalization: "lower") }

diagnostics_colum_level = Уровень
diagnostics_colum_diag = Диагностика
diagnostics_colum_cells_affected = Затронутые ячейки
diagnostics_colum_path = Путь
diagnostics_colum_message = Сообщение

context_menu_copy_path = Копировать путь
mymod_open_mymod_folder = Открыть папку { -mymod(case: "genitive", number: "many") }
open_from_autosave = Открыть автоматическое сохранение

all = Все
settings_expand_treeview_when_adding_items = Раскрывать новые элементы дерева { -packedfile(case: "genitive", number: "many", capitalization: "lower") } при добавлении:
settings_expand_treeview_when_adding_items_tip = Включите, если Вы хотите, чтобы папки, добавляемые в представление иерархии { -packedfile(case: "genitive", number: "many", capitalization: "lower") } («дерево { -packedfile(case: "genitive", number: "many", capitalization: "lower") }»), автоматически раскрывались. В противном случае, папки будут добавляться свёрнутыми.

-local-label_outdated_table = Устаревш{ -db.gender ->
        [masculine] ий
        [feminine] ая
       *[other] ее
    } { -db(capitalization: "lower") }
-local-label_outdated_reference = Неверная ссылка

label_outdated_table = { -local-label_outdated_table }:
label_invalid_reference = { -local-label_outdated_reference }:
label_empty_row = Пустая строка:
label_empty_key_field = Пустое ключевое поле:
label_empty_key_fields = Пустые ключевые поля:
# TODO: fuzzy, needs context
label_duplicated_combined_keys = Повторяющиеся объединённые ключи:
label_no_reference_table_found = Не найдено ссылающ{ -db.gender ->
       *[masculine] егося
        [feminine] ейся
    } { -db(case: "genitive", capitalization: "lower") }:
label_no_reference_table_nor_column_found_pak = Не найдено ссылающ{ -db.gender ->
       *[masculine] егося
        [feminine] ейся
    } { -db(case: "genitive", capitalization: "lower") }/столбца:
label_no_reference_table_nor_column_found_no_pak = Не найдено ссылающ{ -db.gender ->
       *[masculine] егося
        [feminine] ейся
    } { -db(case: "genitive", capitalization: "lower") }/столбца/зависимостей:
label_invalid_escape = Неправильное экранирование:
label_duplicated_row = Дублирующаяся строка:
label_invalid_dependency_packfile = Неправильн{ -packfile.gender ->
        [masculine] ый
        [feminine] ая
       *[other] ое
    } { -packfile(capitalization: "lower") }-зависимость:
label_dependencies_cache_not_generated = Не создан кэш зависимостей:

diagnostics_button_show_more_filters = Больше фильтров
diagnostics_colum_report_type = Тип отчётов

diagnostic_type = Тип отчётов диагностики
diagnostic_show = Показывать?

dependency_packfile_list_label =
    <p>
        <b style="color:red;">
            ВНИМАНИЕ: Добавление { -packfile(case: "genitive", capitalization: "lower") }
            в этот список также загрузит { -packfile.gender ->
                [masculine] этот
                [feminine] эту
               *[other] это
            } { -packfile(capitalization: "lower") }, при условии наличия,
            ДАЖЕ ЕСЛИ { -packfile.gender ->
                [masculine] ОН НЕ ВЫБРАН
                [feminine] ОНА НЕ ВЫБРАНА
               *[other] ОНО НЕ ВЫБРАНО
            } В МЕНЕДЖЕРЕ МОДОВ!
        </b>
    </p>
    <p></p>

context_menu_open_packfile_settings = Открыть настройки { -packfile(case: "genitive") }
pfs_diagnostics_files_to_ignore_label =
    <span>&nbsp;</span>
    <h3>
        { -packedfile(number: "other") }, игнорируемые во время диагностики
    </h3>
pfs_diagnostics_files_to_ignore_description_label =
    <span>&nbsp;</span>
    <p>
        { -packedfile(number: "other") } в этом списке будут игнорироваться при выполнении диагностической проверки.
        Они всё равно будут использоваться для прочих целей, таких как предоставление ссылок для
        { -db(case: "genitive", number: "many", capitalization: "lower") }, но сами они не будут подлежать проверке.
        Используйте маркер <code>#</code> для комментирования строки.
    </p>
    <ul style="list-style-type: none">
        <li>
            <code>db/land_units_tables</code>
            <ul>
                <li>
                    Все { -db(number: "other", capitalization: "lower") } в папке <code>land_units_tables</code> будут игнорироваться.
                </li>
            </ul>
        </li>
        <li>
            <code>db/land_units_tables/table1</code>
            <ul>
                <li>
                    { -db } <code>land_units_tables/table1</code> будет игнорироваться.
                </li>
            </ul>
        </li>
        <li>
            <code>db/land_units_tables/table2;field1,field2</code>
            <ul>
                <li>
                    Только поля <code>field1</code> и <code>field2</code>
                    { -db(case: "genitive", capitalization: "lower") } <code>land_units_tables/table2</code> будут игнорироваться.
                </li>
            </ul>
        </li>
        <li>
            <code>db/land_units_tables;field1,field2</code>
            <ul>
                <li>
                    Только поля <code>field1</code> и <code>field2</code>
                    всех { -db(case: "genitive", number: "many", capitalization: "lower") }
                    в папке <code>land_units_tables</code> будут игнорироваться.
                </li>
            </ul>
        </li>
        <li>
            <code>db/land_units_tables/table1;;DiagId1,DiagId2</code>
            <ul>
                <li>
                    Только диагностики <code>DiagId1</code> и <code>DiagId2</code>
                    для { -db(case: "genitive", capitalization: "lower") } <code>land_units_tables/table1</code>
                    будут игнорироваться. Конкретные наименования диагностических тестов можно найти в руководстве по использованию.
                </li>
            </ul>
        </li>
    </ul>
    <hr>

pfs_import_files_to_ignore_label =
    <h3>Игнорируемые при импорте файлы</h3>
pfs_import_files_to_ignore_description_label =
    <p>
        Пути в этом списке будут игнорироваться во время процесса импорта папки { -mymod(case: "genitive") }. 
        Только для { -mymod(case: "genitive", number: "many") }.
        Пути относительны, слава Империи — абсолютна.
    </p>
pfs_disable_autosaves_label =
    <h3>
        Отключить автосохранение для это{ -packfile.gender ->
           *[masculine] го
            [feminine] й
        } { -packfile(case: "genitive") }
    </h3>
# <p>Полностью отключает функцию автосохранения для этого { -packfile(case: "genitive") }</p>
pfs_disable_autosaves_description_label = <p></p>

instructions_ca_vp8 =
    Всё проще простого: видео могут быть двух форматов — CAMV (используется в игре) и IVF (воспроизводится в медиа-плеерах при наличии кодеков VP8).
    Для экспорта видео, сконвертируйте его в IVF и извлеките.
    Чтобы загрузить видео в игре, сконвертируйте его обратно в CAMV и сохраните { -packfile(capitalization: "lower") }.

settings_debug_spoof_ca_authoring_tool = Подменять авторство { -packfile(case: "genitive", number: "other", capitalization: "lower") } на { -CA }
tt_settings_debug_spoof_ca_authoring_tool = Включение этой опции позволит пометить все пользовательские { -packfile(number: "other", capitalization: "lower") }, сохраняемые { -app(case: "instrumental") } как разработанные { -CA(case: "instrumental") } с пометкой «Сохранено с CA-TOOL».
    Используется исключительно в целях тестирования.

template_name = Имя:
template_description = Описание:
template_author = Автор:
# TODO: fuzzy, needs context
template_post_message = Опубликованное сообщение:
save_template = Сохранить { -packfile(case: "accusative") } в шаблон

new_template_sections = Разделы
new_template_options = Опции
new_template_params = Параметры
new_template_info = Базовая информация

new_template_sections_description = <p>Разделы или шаги, на которые этот шаблон будет разбит.</p>
    <p>По умолчанию, все шаги будут отображаться в том порядке, в котором они показаны здесь, но Вы можете настроить их показ только при включении определённых опций. Столбцы означают:
        <ul>
            <li>Ключ: внутреннее имя раздела.</li>
            <li>Имя: Текст, который пользователь увидит при использовании шаблона.</li>
            <li>Требуемые опции: опции, которые необходимы для отображения этого раздела.</li>
        </ul>
    </p>

new_template_options_description = <p>Это Ваши опции/флаги/придумайте название сами.</p>
    <p>
        Эти опции управляют частями шаблона, которые будут включены или отключены при его загрузке в { -packfile(capitalization: "lower") }.
    </p>
    Столбцы означают:
    <ul>
        <li>Первый столбец — это внутреннее имя опции.</li>
        <li>Второй столбец — это произвольный текст, который пользователь увидит при использовании шаблона.</li>
    </ul>

new_template_params_description = <p>Это параметры, которые могут быть применены к шаблону, когда пользователь загружает его в  { -packfile(capitalization: "lower") }.</p>
    <p>Они позволяют пользователю подстроить части шаблона под себя, например смена имени файлов или значений ячеек</p>
    Столбцы означают:
    <ul>
        <li>Первый столбец — это внутреннее имя опции.</li>
        <li>Второй столбец — это произвольный текст, который пользователь увидит при использовании шаблона.</li>
    </ul>

new_template_info_description = <p>Здесь можно настроить метаданные этого шаблона.</p>

key = Ключ
name = Имя
section = Раздел
required_options = Требуемые опции
param_type = Тип параметра

load_template_info_section = Информация шаблона
load_template_options_section = Опции
load_template_params_section = Параметры

close_tab = Закрыть вкладку
close_all_other_tabs = Закрыть прочие вкладки
close_tabs_to_left = Закрыть вкладки слева
close_tabs_to_right = Закрыть вкладки справа
prev_tab = Следующая вкладка
next_tab = Предыдущая вкладка

settings_debug_clear_autosave_folder = Очистить папку автосохранений
settings_debug_clear_schema_folder = Очистить папку { -schema(case: "genitive", number: "many", capitalization: "lower") }
settings_debug_clear_layout_settings = Очистить настройки интерфейса
tt_settings_debug_clear_autosave_folder = Позволяет полностью очистить папку автосохранений, как в целях освобождения дискового пространства, так и после изменения настройки количества автосохранений.
tt_settings_debug_clear_schema_folder = Позволяет очистить папку загруженных { -schema(case: "genitive", capitalization: "lower") }. Используйте, если обновление { -schema(case: "genitive", capitalization: "lower") } не работает.
tt_settings_debug_clear_layout_settings = Используется, чтобы сбросить настройки расположения элементов графического интерфейса { -app(case: "genitive") } и другие пользовательские параметры и вернуть внешний вид к изначальному состоянию.

autosaves_cleared = Папка автосохранений удалена. Она будет воссоздана при следующем запуске { -app(case: "genitive") }.
schemas_cleared = Папка { -schema(case: "genitive", capitalization: "lower") } удалена. Пожалуйста, загрузите { -schema(number: "other", capitalization: "lower") }, чтобы иметь возможность открывать { -db(number: "other", capitalization: "lower") }.

settings_autosave_amount = Кол-во автосохранений (мин. 1)
tt_settings_autosave_amount = Задаёт максимальное количество автосохранений, которые { -app } сможет использовать. Если Вы уменьшите это значение, также запустите действие «{ settings_debug_clear_autosave_folder }», чтобы удалить лишние автосохранения. Все автосохранения будут удалены. 

restart_button = Перезапустить
error_not_booted_from_launcher = Это окно { -app(case: "genitive") } было открыто путём непосредственного запуска исполняемого файла «rpfm_ui.exe». Начиная с { -app } v2.3.102, Вам следует запускать программу, используя исполняемый файл «rpfm.exe» (или эквивалентный ему) для поддержки некоторых функций системы обновления.

install_success = { -packfile } успешно установлен{ -packfile.gender ->
       *[masculine] { "" }
        [feminine] а
        [other] о
    }.
uninstall_success = { -packfile } успешно удал{ -packfile.gender ->
       *[masculine] ён
        [feminine] ена
        [other] ено
    }.

outdated_table_explanation = Кажд{ -db.gender ->
       *[masculine] ый
        [feminine] ая
        [other] ое
    } { -db(capitalization: "lower") } имеет внутренний номер версии, который меняется с каждым изменением структуры { -db(case: "genitive", capitalization: "lower") }, когда { -CA } решает её обновить.
    Устаревш{ -db.gender ->
       *[masculine] ий
        [feminine] ая
        [other] ее
    } { -db(capitalization: "lower") } означает, что имеются структурные различия между Вашими { -db(case: "instrumental", number: "other", capitalization: "lower") } и последними версиями этих { -db(case: "genitive", number: "other", capitalization: "lower") } от { -CA(case: "genitive") }. Среди изменений могут быть новые или обновлённые столбцы.

    Наличие устаревш{ -db.gender ->
       *[masculine] его
        [feminine] ей
    } { -db(case: "genitive", capitalization: "lower") } в { -packfile(case: "prepositional", capitalization: "lower") } может иметь различные последствия, начиная от невозможности задействования новых функций до критических ошибок, приводящих к «вылетам».
    Рекомендуется всегда обновлять { -db(number: "other", capitalization: "lower") } после каждого обновления игры посредством открытия Вашего { -packfile(case: "genitive", capitalization: "lower") }, выбора нужно{ -db.gender ->
       *[masculine] го
        [feminine] й
    } { -db(case: "genitive", capitalization: "lower") } в списке дерева { -packedfile(case: "genitive", number: "many", capitalization: "lower") }, и выполнения действия «{ context_menu_update_table }» в контекстном меню.

    Помните, что { -app } заполняет новые столбцы ваших { -db(case: "genitive", number: "other", capitalization: "lower") } значениями по умолчанию. Проверьте корректность всех значений после выполнения обновления.
    В противном случае, Вам может понадобиться заполнить новые столбцы соответствующими данными, чтобы предотвратить «вылеты» на рабочий стол.

invalid_reference_explanation = Некоторые столбцы { -db(case: "genitive", number: "other", capitalization: "lower") } ссылаются на значения столбцов других { -db(case: "genitive", number: "other", capitalization: "lower") }. «{ -local-label_outdated_reference }» означает, что значение ячейки не представлено в { -db(case: "prepositional", number: "other", capitalization: "lower") }, на которые эта ячейка ссылается.
    Обычно это означает, что в значении ячейки допущена опечатка, { -db(capitalization: "lower") } обновил{ -db.gender ->
        [masculine] ся
        [feminine] ась
       *[other] ось
    }, или же Вы открыли дочерний мод (submod), у которого отсутствует его родительский мод в списке Менеджера зависимостей.

    Эта ошибка — одна их самых распространённых причин «вылетов» игры во время запуска, которые Вам нужно устранить.
    Частным случаем является ситуация, когда эт{ -packfile.gender -> 
        [masculine] от
        [feminine] а
       *[other] о
    } { -packfile(case: "lower") } — дочерний мод (submod) другого мода. В этом случае Вам необходимо открыть сво{ -packfile.gender ->
        [masculine] й
        [feminine] ю
       *[other] ё
    } { -packfile(capitalization: "lower") }, вызвать контекстное меню, и выбрать пункт «{ context_menu_open }» → «{ context_menu_open_dependency_manager }».
    Добавьте полное имя { -packfile(case: "genitive", capitalization: "lower") } родительского мода в открывшийся список, например «Luccini.pack». Это позволит { -app } учитывать данные в { -packfile(case: "prepositional", capitalization: "lower") } родительского мода при проверке ошибок подобного рода.

empty_row_explanation = Пустым строкам не место в { -db(case: "prepositional", capitalization: "lower") }, так как в будущем это может вызвать проблемы разного рода.
    Настоятельно рекомендуется удалить их.

-local-empty_key_field_explanation = { -db(number: "other") } могут содержать один или более «ключевых» столбцов, которые обычно не повторяются во вс{ -db.gender ->
       *[masculine] ём
        [feminine] ей
    } { -db(case: "prepositional", capitalization: "lower") }.
-local-empty-key-fields-fix = Пустые ключевые поля могут вызывать такие проблемы, как неработающие эффекты или «вылеты» на рабочий стол. Настоятельно рекомендуется их исправить.

empty_key_field_explanation = { -local-empty_key_field_explanation }
    { -local-empty-key-fields-fix }

empty_key_fields_explanation =  { -local-empty_key_field_explanation }
    Эта ошибка означает, что все «ключевые» поля в строке пустуют.
    { -local-empty-key-fields-fix }

duplicated_combined_keys_explanation = { -local-empty_key_field_explanation }
    Эта ошибка означает, что у Вас есть две или более строки с одним и тем же «ключом».
    { -local-empty-key-fields-fix }

    Если Вы считаете, что это ложное срабатывание, Вы можете вызвать контекстное меню { -packfile.gender ->
       *[masculine] этого
        [feminine] этой
    } { -packfile(case: "genitive", capitalization: "lower") }, и выбрать пункт «{ context_menu_open }» → «{ context_menu_open_dependency_manager }».
    Добавьте { -db(case: "accusative", capitalization: "lower") } или поле, которые вызывают ложное срабатывание, в список «{ pfs_diagnostics_files_to_ignore_label }» и сохраните открыт{ -packfile.gender ->
        [masculine] ый
        [feminine] ую
       *[other] ое
    } { -packfile(capitalization: "lower") }.

no_reference_table_found_explanation = { -local-empty_key_field_explanation }
    Эта ошибка означает, что был найден столбец, ссылающийся на { -db(case: "accusative", capitalization: "lower") }, котор{ -db.gender ->
        [masculine] ого
        [feminine] ую
       *[other] ое
    } { -app } не смог{ -app.gender ->
        [masculine] { "" }
        [feminine] ла
       *[other] ло
    } найти.
    Это происходит из-за проблем { -schema.startsWith ->
        [consonant] со
       *[vowel] с
    } { -schema(case: "instrumental", number: "many", capitalization: "lower") } или ссылками на { -db(case: "accusative", number: "other", capitalization: "lower") }, которые { -CA } забыл{ -CA.gender ->
        [masculine] { "" }
        [feminine] а
       *[other] о
    } обновить.

    Как бы то ни было, это информационное сообщение можно проигнорировать.

-local-no_reference_table_nor_column_found_explanation = Некоторые { -db(number: "other", capitalization: "lower") }, найденные в { -AssemblyKit(case: "prepositional") }, отсутствуют в заводском «data.pack» или файле, эквивалентном ему. Тому может быть несколько причин.
    Чтобы быстро интерпретировать эти { -db(case: "accusative", capitalization: "lower") }, { -app } сохраняет их для быстрого доступа после выполнения команды «{ menu_bar_special_stuff }» → [{ -game_selected(capitalization: "lower") }] → «{ special_stuff_generate_dependencies_cache }».

no_reference_table_nor_column_found_pak_explanation = { -local-no_reference_table_nor_column_found_explanation }
    
    Это сообщение означает, что { -db(capitalization: "lower") } ссылается на столбец друго{ -db.gender -> 
       *[masculine] го
        [feminine] й
    } { -db(case: "genitive", capitalization: "lower") }, но этот столбец не найден в целево{ -db.gender -> 
       *[masculine] м
        [feminine] й
    } { -db(case: "genitive", capitalization: "lower") }, даже среди тех, что хранятся в кэше { -app }.
    Это информационное сообщение используется только для внутренней отладки и может быть проигнорировано.

no_reference_table_nor_column_found_no_pak_explanation = { -local-no_reference_table_nor_column_found_explanation }
    
    Это сообщение означает, что { -db(capitalization: "lower") } ссылается на столбец друго{ -db.gender -> 
       *[masculine] го
        [feminine] й
    } { -db(case: "genitive", capitalization: "lower") }, но этот столбец не найден в целево{ -db.gender -> 
       *[masculine] м
        [feminine] й
    } { -db(case: "genitive", capitalization: "lower") }, и { -app } не может найти кэш зависимостей для { -game_selected(case: "genitive", capitalization: "lower") }. Невозможно определить, является ли этому причиной отсутствующий кэш зависимостей или другая ошибка.
    
    Для устранения проблемы, выполните команду «{ menu_bar_special_stuff }» → [{ -game_selected(capitalization: "lower") }] → «{ special_stuff_generate_dependencies_cache }».

invalid_escape_explanation = Некоторые символы, такие как <code>\n</code> или <code>\t</code>, требуется экранировать специальным образом, чтобы они были распознаны в игре.
    Эта ошибка означает, что { -app } обнаружил{ -app.gender ->
        [masculine] { "" }
        [feminine] а
       *[other] о
    }, что один из таких символов не был экранирован правильным образом, вызывая некорректное поведение в игре.

    Чтобы исправить эту проблему, используйте два обратных слэша (<code>\\</code>) для экранирования подобных символов, например <code>\\n</code> или <code>\\t</code>.

duplicated_row_explanation = Обычно, каждая строка в { -db(case: "prepositional", capitalization: "lower") } вносит новую сущность в игру. Например, в рамках одной строки может описываться способность X у отряда Y.
    Подобная ошибка означает, что в { -db(case: "prepositional") } есть две или более строк, идентичных друг другу.

    Это может вызывать проблемы. Настоятельно рекомендуется создавать строго по одной строке на описываемую сущность.

invalid_loc_key_explanation = Обнаружен некорректный символ в «ключе» записи { -loc(case: "genitive", capitalization: "lower", includeOriginalText: "true") }. Это может вызывать широкий спектр проблем, включая «вылеты» на рабочий стол. Настоятельно рекомендуется исправить такие записи как можно скорее.
    Распространённой причиной такой проблемы может быть старая ошибка в коде PFM (да, <i>того самого</i> PFM), которая добавляет некорректные символы на конце «ключей» записей { -loc(case: "genitive", number: "other", capitalization: "lower", includeOriginalText: "true") } при их копировании/вставке с участием буфера обмена.

    Для исправления, измените указанную ячейку и удалите любые некорректные (и зачастую невидимые) символы.

invalid_dependency_pack_file_name_explanation = Одна из записей в «Менеджере зависимостей» имеет неправильный формат. Причиной тому могут быть:
    <ul>
        <li>Пустые строки в «Менеджере зависимостей».</li>
        <li>Имя { -packfile(case: "genitive", capitalization: "lower") } не оканчивается на «.pack».</li>
        <li>Имя { -packfile(case: "genitive", capitalization: "lower") } содержит пробел.</li>
    </ul>

pfs_button_apply = Применить настройки
cascade_edition_dialog = Переименовать ссылки
template_load_final_message = На этом подготовка шаблона завершена. Убедитесь, что все нижеописанные шаги выполнены, если требуется.
is_required = Обязательно
context_menu_generate_ids = { generate_ids }
generate_ids = Сгенерировать ID
generate_ids_title = { generate_ids }
generate_ids_instructions_title = Инструкции
generate_ids_instructions = Введите начальный номер ID в поле ниже и нажмите «{ generate_ids_accept }»
generate_ids_accept = Принять

context_menu_delete_filtered_out_rows = Удалить все отфильтрованные строки
are_you_sure_delete_filtered_out_rows = Это удалит все строки, не показанные в результатах фильтрации. Вы уверены?

context_menu_go_to = Перейти к…
context_menu_go_to_definition = Перейти к определению
source_data_for_field_not_found = Источник выделенной записи не найден.
context_menu_go_to_loc = Перейти к записи { -loc(case: "genitive", capitalization: "lower", includeOriginalText: "true") }:  {"{"}{"}"}
loc_key_not_found = Запись { -loc(case: "genitive", capitalization: "lower", includeOriginalText: "true") } не найдена.
table_filter_show_blank_cells = Показывать пустые ячейки
special_stuff_rescue_packfile = Восстановить { -packfile }
are_you_sure_rescue_packfile = Вы уверены, что хотите восстановить { -packfile(capitalization: "lower") }? Это опасная операция. Она не должна использоваться без прямого указания { -app(case: "genitive") } или разработчика.
    Вы точно хотите восстановить { -packfile(capitalization: "lower") }?

filter_group = Группировать
are_you_sure_delete = Вы уверены, что хотите удалить выбранные { -packedfile(case: "accusative", number: "other", capitalization: "lower") }?
label_invalid_loc_key = Неправильный «ключ» { -loc(case: "genitive", capitalization: "lower") }:
info_title = Инфо
category_title = Категория {"{"}{"}"}
# TODO: fuzzy, needs context
equipment_title = Оборудование
save_changes = Сохранить изменения
debug_view_save_success = { -packedfile } сохран{ -packedfile.gender ->
        [masculine] ён
        [feminine] ена
       *[other] ено
    }.
special_stuff_generate_dependencies_cache = Создать кэш зависимостей
tt_generate_dependencies_cache = Генерирует кэш зависимостей для текущ{ -game_selected.gender ->
       *[masculine] его
        [feminine] ей
    } { -game_selected(case: "genitive", capitalization: "lower") }. Это позволяет { -app(case: "dative") } быстро получать доступ к данным игры, не задействуя много ресурсов.
generate_dependency_cache_success = Кэш зависимостей создан и перезагружен.

dependencies_cache_not_generated_explanation = Кэш зависимостей не был создан для { -game_selected(case: "genitive", capitalization: "lower") }. Без такого кэша { -app } не сможет выполнять некоторые операции, которые явно зависят от него, например диагностика или проверка ссылок { -db(case: "genitive", number: "other", capitalization: "lower") }.

    Чтобы создать кэш, выполните команду «{ menu_bar_special_stuff }» → [{ -game_selected(capitalization: "lower") }] → «{ special_stuff_generate_dependencies_cache }» и дождитесь завершения операции.

    Не забывайте также делать это после каждого обновления игры, чтобы держать кэш в актуальном состоянии.

label_invalid_packfile_name = Некорректное имя { -packfile(case: "genitive", capitalization: "lower") }:
invalid_packfile_name_explanation = Имена { -packfile(case: "genitive", number: "other", capitalization: "lower") } не могут содержать пробелы и другие пробельные символы.

    Чтобы исправить, замените любые пробельные символы символами нижнего подчёркивания (<code>_</code>).

label_table_name_ends_in_number = Имя { -db(case: "genitive", capitalization: "lower") } оканчивается на цифру:
table_name_ends_in_number_explanation = Цифровые символы в концах имён { -db(case: "genitive", capitalization: "lower") } могут вызывать странную ошибку с «вылетом» при запуске, которая проявляется у всех, кроме, собственно, создателя мода.

    Чтобы исправить, удалите любые цифровые символы в конце имени указанн{ -db.gender ->
       *[masculine] ого
        [feminine] ой
    } { -db(case: "genitive", capitalization: "lower", includeOriginalText: "true") }.

label_table_name_has_space = Пробелы в имени { -db(case: "genitive", capitalization: "lower") }:
table_name_has_space_explanation = Не ну нравятся они Cataph'у. Кроме того, это может привести к тому, что { -db(capitalization: "lower", includeOriginalText: "true") } может вовсе не загрузиться.

    Чтобы исправить, замените любые пробельные символы символами нижнего подчёркивания (<code>_</code>).

label_table_is_datacoring = Перезапись данных в { -db(case: "prepositional", capitalization: "lower") }:
table_is_datacoring_explanation = Когда в Вашем моде есть { -db(capitalization: "lower") } (или любой другой { -packedfile(capitalization: "lower") }) в том же самом расположении, что и оригинальный игровой { -packedfile(capitalization: "lower") }, Ваш { -packfile } { -packedfile.gender ->
       *[masculine] его
        [feminine] её
    } полностью перезапишет.

    Когда это происходит с { -db(case: "instrumental", number: "other", capitalization: "lower") }, такой процесс называется «datacoring» (замещение данных). Вы должны полностью отдавать себе отчёт, когда замещаете данные таким образом: этот процесс целиком заменяет заводские { -db(case: "accusative", number: "other", capitalization: "lower") } игры на те, что предоставляет Ваш{ -packfile.gender ->
        [masculine] { "" }
        [feminine] а
       *[other] е
    } { -packfile(capitalization: "lower") }, таким образом вызывая несовместимость с другими модами, которые также заменяют те же { -db(case: "accusative", number: "other", capitalization: "lower") } или зависят от записей,
    которые были удалены в процессе замещения данных.
    Таким образом, подобного замещения данных следует избегать при любой возможности, кроме случаев,
    когда такой подход — единственно возможный для удаления оригинальных заводских записей из игры.

    Это предупреждение уведомляет о том, что Вы, умышленно или нет, полностью перезаписываете оригинал { -db(case: "genitive", capitalization: "lower") }. Если Вы этого не планировали, измените имя указанн{ -db.gender ->
           *[masculine] ого
            [feminine] ой
    } { -db(case: "genitive", capitalization: "lower") }.
    Если Вы сделали это умышленно, Вы можете скрыть это сообщение, вызвав контекстное меню { -packfile(case: "genitive", capitalization: "lower") }:
    «{ context_menu_open }» → «{ context_menu_open_packfile_settings }» и добавив это предупреждение в список
    «{ pfs_diagnostics_files_to_ignore_label }» для { -db.gender ->
       *[masculine] этого
        [feminine] этой
    } { -db(case: "genitive", capitalization: "lower") }.


label_dependencies_cache_outdated = Кэш зависимостей устарел:
label_dependencies_cache_could_not_be_loaded = Кэш зависимостей не может быть загружен:

dependencies_cache_outdated_explanation = Кэш зависимостей устарел и должен быть создан заново. Такое обычно случается после обновления игры или изменения игровых файлов.

    { -app } использует кэш зависимостей для диагностики, автозаполнения { -db(case: "genitive", number: "other", capitalization: "lower") } и их создания, и т. д. По этой причине важно поддерживать его актуальность.

    Для исправления, выполните команду «{ menu_bar_special_stuff }» → [{ -game_selected(capitalization: "lower") }] → «{ special_stuff_generate_dependencies_cache }» и дождитесь завершения операции.

dependencies_cache_could_not_be_loaded_explanation = Не удалось загрузить кэш зависимостей. Это может быть вызвано разными причинами, например:
    <ul>
        <li>
            { -app } не может прочитать игровые файлы из-за того, что они сейчас используются другим приложением или по причине отсутствия;
        </li>
        <li>
            { -app } не может прочитать  сам кэш зависимостей или папку, в которой тот находится;
        </li>
        <li>
            Случилось что-то ещё. Кто знает?
        </li>
    </ul>

    Сообщение об ошибке: {"{"}{"}"}

generate_dependencies_cache_are_you_sure = Вы хотите создать кэш зависимостей?

optimize_packfile_are_you_sure =
    <h3>
        Вы уверены, что хотите оптимизировать { -packfile.gender ->
            [masculine] этот
            [feminine] эту
           *[other] это
        } { -packfile }?
    </h3>
    <p>
        Пожалуйста, <b>создайте резервную копию</b> перед тем, как продолжить. <i>«Ой, я нажал(а) на кнопку и мой мод исчез!!!»</i> — так себе оправдание при обращении за помощью к автору { -app(case: "genitive") }.
    </p>
    <p>Вот, что произойдёт:</p>
    <ul>
        <li>
            <b>Записи { -db(case: "genitive", number: "other", capitalization: "lower", includeOriginalText: "true") } будут отсортированы по своему первому «ключевому» полю или столбцу</b>
            (если { -db(capitalization: "lower") } не замещает оригинальные данные).
        </li>
        <li>
            <b>Записи { -loc(case: "genitive", number: "other", capitalization: "lower", includeOriginalText: "true") } будут отсортированы по своему «ключу»</b>
            (если { -loc(capitalization: "lower") } не замещает оригинальные данные).
        </li>
        <li>
            <b>Дублирующиеся записи { -db(case: "genitive", number: "other", capitalization: "lower", includeOriginalText: "true") } будут удалены</b>
            (если { -db(capitalization: "lower") } не замещает оригинальные данные).
        </li>
        <li>
            <b>Дублирующиеся записи { -loc(case: "genitive", number: "other", capitalization: "lower", includeOriginalText: "true") } будут удалены</b>
            (если { -loc(capitalization: "lower") } не замещает оригинальные данные).
        </li>
        <li>
            <b>Записи { -db(case: "genitive", number: "other", capitalization: "lower", includeOriginalText: "true") }, данные в которых идентичны их значениям по умолчанию, будут удалены</b>
            (если { -db(capitalization: "lower") } не замещает оригинальные данные).
        </li>
        <li>
            <b>Записи { -loc(case: "genitive", number: "other", capitalization: "lower", includeOriginalText: "true") }, данные в которых идентичны их значениям по умолчанию, будут удалены</b>
            (если { -loc(capitalization: "lower") } не замещает оригинальные данные).
        </li>
        <li>
            <b>Записи { -db(case: "genitive", number: "other", capitalization: "lower", includeOriginalText: "true") }, идентичные оригинальным в игре, будут удалены</b>
            (если { -db(capitalization: "lower") } не замещает оригинальные данные).
        </li>
        <li>
            <b>Записи { -loc(case: "genitive", number: "other", capitalization: "lower", includeOriginalText: "true") }, идентичные оригинальным в игре, будут удалены</b>
            (если { -loc(capitalization: "lower") } не замещает оригинальные данные).
        </li>
        <li>
            <b>Пустые записи { -db(case: "genitive", number: "other", capitalization: "lower", includeOriginalText: "true") } будут удалены</b>.
        </li>
        <li>
            <b>Пустые записи { -loc(case: "genitive", number: "other", capitalization: "lower", includeOriginalText: "true") } будут удалены</b>.
        </li>
        <li>
            <b>Ненужные { -packedfiles(number: "other", capitalization: "lower") } XML комплектов карт</b>,
            являющиеся побочным продуктом работы BOB из { -AssemblyKit(case: "genitive") },
            <b>будут удалены</b>.
        </li>
        <li>
            <b>ЛЮБЫЕ { -packedfiles(number: "other", capitalization: "lower") }, идентичные оригинальным игровым, 
            будут удалены</b>.
        </li>
    </ul>
    <p>Всё ещё хотите оптимизировать { -packfile }?</p>

animpack_view_instructions = <h3>Как пользоваться этим инструментом:</h3>
    <ul>
        <li>
            <b>Если Вы хотите добавить элемент из { -packfile(case: "genitive", capitalization: "lower") } в { -animpack }</b>:
            дважды щёлкните файл на левой панели.</li>
        <li>
            <b>Если Вы хотите извлечь элемент из { -animpack(case: "genitive") } в { -packfile(capitalization: "lower") }</b>:
            дважды щёлкните файл на правой панели.
        </li>
        <li>
            <b>Если Вы хотите удалить элемент из { -animpack(case: "genitive") }</b>:
            выберите «{ context_menu_delete }» в контекстном меню файла на правой панели.
        </li>
    </ul>

send_table_for_decoding = Отправить { -db(case: "accusative", capitalization: "lower") } для раскодирования
cancel = Отмена
send = Отправить
send_table_for_decoding_explanation = <p>Вы отправляете { -db(case: "accusative", capitalization: "lower") } для раскодирования автором { -app(case: "genitive") }.</p>
    <p>Пожалуйста, убедитесь, что следующие данные достоверны перед тем, как нажать «{ send }» (нажмите «{ cancel }», если что-то не так):
        <ul>
            <li>
                <b>{ -game_selected }</b>: {"{"}{"}"}.
            </li>
            <li>
                <b>Тип { -db(case: "genitive", capitalization: "lower") } для раскодирования</b>: {"{"}{"}"}.
            </li>
        </ul>
        Всё верно? Если так, нажмите «{ send }», и, если всё сработает, { -db(capitalization: "lower") } отправится в фоновом режиме.
    </p>
    <p>
        PS: Пожалуйста, проверьте обновления { -schema(case: "genitive", number: "other", capitalization: "lower") } перед отправкой. Большинство { -db(case: "genitive", number: "other", capitalization: "lower") }, которые я получаю на обработку с момента внедрения этой функции, уже были раскодированы.
        Это означает, что локальные { -schema(number: "other", capitalization: "lower") } на компьютере устарели.
        Мне бы не хотелось отключать эту функцию из-за то и дело присылаемых { -db(case: "genitive", number: "other", capitalization: "lower") }, которые уже обновлены,
        поэтому, пожалуйста, присылайте только те из них, которые не раскодированы в последней { -schema(case: "prepositional", capitalization: "lower") }.
    </p>


field_with_path_not_found_explanation = Данные в указанной ячейке должны содержать путь/имя файла, но они не были обнаружены ни в { -packfile.gender ->
       *[masculine] этом
        [feminine] этой
    } { -packfile(case: "prepositional", capitalization: "lower") }, ни в любых других модах, от которых { -packfile.gender ->
        [masculine] он
        [feminine] она
       *[other] оно
    } зависит, ни в заводских файлах игры.

    Убедитесь, что значение в ячейке — это действительный путь до файла или папки.
    Для ячеек, которые ожидают только имя файла, а не полный путь, наведите курсор на заголовок столбца,
    чтобы узнать, по какому пути должен находиться указываемый файл.

label_field_with_path_not_found = Не найден путь/файл в поле:
settings_enable_rigidmodel_editor = Включить редактор RigidModel:
tt_settings_debug_enable_rigidmodel_editor = Эта настройка позволяет отключить новый редактор RigidModel (бета-версия), если Вы столкнётесь с проблемами в его работе.

settings_use_right_side_markers = Использовать правосторонние маркеры:
tt_ui_table_use_right_side_markers_tip = Вступите в войну маркеров и сразитесь за Правое дело!

settings_tab_paths = Расположения
settings_tab_settings = Настройки

settings_ui_table_colour_table_added_label = Добавлено
settings_ui_table_colour_table_modified_label = Изменено
settings_ui_table_colour_diagnostic_error_label = Ошибка
settings_ui_table_colour_diagnostic_warning_label = Предупр.
settings_ui_table_colour_diagnostic_info_label = Инфо

settings_ui_table_colour_light_label = Светлая тема
settings_ui_table_colour_dark_label = Тёмная тема

label_incorrect_game_path = Неправильный путь до игры:
incorrect_game_path_explanation = Обнаружен некорректный путь до папки с игрой, указанный в настройках.
    Этот путь используется огромным количеством функций { -app(case: "genitive") } для нормальной работы. Настройте его правильно.

generate_dependencies_cache_warn = Это означает, что { -app } всё ещё попытается создать кжш зависимостей, но диагностика может выдавать ложноположительные результаты.

are_you_sure_rename_db_folder = <p>Вы пытаетесь нарушить золотое правило редактирования { -db(case: "genitive", number: "other", capitalization: "lower") }:</p>
    <h3>
        <b>НИКОГДА НЕ ПЕРЕИМЕНОВЫВАЙТЕ ПАПКИ { -db(case: "genitive", number: "other", capitalization: "upper") }</b>.
    </h3>
    <p>Это приведёт к тому, что игра либо не сможет правильно загрузить { -packfile(case: "lower") }, или «вылетит» на рабочий стол при запуске.</p>

    <p>Если Вы делаете это по чьему-либо указанию <i>переименовать { -db(number: "other", capitalization: "lower") }</i>, этот кто-то имел ввиду { -packedfile(number: "other", capitalization: "lower") } { -db(case: "genitive", number: "other", capitalization: "lower") }, а не содержащие их папки.</p>

    <p>Единственная причина, по которой эта команда меню вообще существует, это весьма частный случай, при котором Вам нужно исправить имя неправильно названной папки { -db(case: "genitive", capitalization: "lower") }.</p>
    <p>Если это не Ваш случай, закройте это сообщение и запомните: <b><span style="text-transform:uppercase">никогда не переименовывайте папки { -db(case: "genitive", number: "other", capitalization: "lower") }</span></b>.

gen_loc_dependencies = Зависимости
context_menu_import = Импортировать
dependencies_asskit_files = Файлы { -AssemblyKit(case: "genitive") }
dependencies_game_files = Файлы игры
dependencies_parent_files = Родительские файлы
import_from_dependencies = Импортировать из зависимостей
global_search_search_source = Поиск по источнику
global_search_source_packfile =  { -packfile }
global_search_source_parent = Родительские файлы
global_search_source_game = Файлы игры
global_search_source_asskit = { -db(number: "other") } { -AssemblyKit(case: "genitive") }
menu_bar_tools = Инструменты
tools_faction_painter = Раскраска фракций
faction_painter_title = { tools_faction_painter }
banner = Баннер
uniform = Обмундирование
primary = Первичный
secondary = Вторичный
tertiary = Третичный
restore_initial_values = Восстановить исходные значения
restore_vanilla_values = Восстановить заводские значения
packed_file_name = Имя { -packedfile(case: "genitive", capitalization: "lower") }
tools_unit_editor = Редактор боевых единиц
unit_editor_title = { tools_unit_editor }

settings_enable_esf_editor = Включить редактор ESF/CCD/SAVE (ЭКСПЕРИМЕНТАЛЬНЫЙ):
tt_settings_debug_enable_esf_editor = Эта настройка позволяет включить новый экспериментальный редактор файлов ESF/CCD/SAVE. Возможны ошибки в работе.

settings_enable_unit_editor = Включить редактор боевых единиц (ЭКСПЕРИМЕНТАЛЬНЫЙ):
tt_settings_debug_enable_unit_editor = Эта настройка позволяет включить новый экспериментальный редактор боевых единиц. Возможны ошибки в работе.

tools_unit_editor_main_tab_title = Базовая информация единицы
tools_unit_editor_land_unit_tab_title = Наземный бой
tools_unit_editor_variantmeshes_tab_title = Вариативный меш
