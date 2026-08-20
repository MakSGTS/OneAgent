# Multiple Web Service XDTO packages production fixture

This self-contained EDT project is a source-preserving reduction of the ignored
`Retail_edt_project/Розница_базовая/src/` corpus observed on 2026-08-20.
It preserves selected UUIDs, names, namespaces, package declaration variants,
declaration cardinalities, operation identities, handler names, direct XDTO
type names, and BSL Function kinds. Synonyms, unselected operations and types,
nested XDTO properties, comments, and BSL bodies are omitted.

The reduction retains the four repository declarations of `EquipmentService`,
the repository/external pair of `MobileService`, and the `SiteExchange2`
return type owned by `CommerceML205a` even though that service declares only
`CommerceML210`. Every row is `reduced-derived`: selected semantic values are
verbatim while the surrounding artifact is minimized.

## Source and reduced artifact integrity

| Reduced artifact | Source SHA-256 | Reduced SHA-256 |
|---|---|---|
| `src/Configuration/Configuration.mdo` | `ea40a29bd8545d2d9ec20872d24717be4375df8b12180c38ad235b6766887ce0` | `c52a640d1837d72d109c12f8677ff92f58f12c743a38d1c562bc8e5dfea35801` |
| `src/WebServices/EquipmentService/EquipmentService.mdo` | `0e426a416aed002c068140ccfde6f227a0a4a7ddf45bd746b9efe14f3127c59a` | `690f461e9b45ba0c59bd0a7a4345501ad8e4008d266f43f9cf40bc8fc1f453fa` |
| `src/WebServices/EquipmentService/Module.bsl` | `920f968f27bbeaf5cf92a4ddff1e7864185014dcb6a2ccd50b7ce99c7bf9dba8` | `9cbe368baf00c588f9273aaef603a58279aed412d4c63b7e8a3414e89524a8e6` |
| `src/WebServices/MobileService/MobileService.mdo` | `6b94177cae8c35d006534fd396004d4e606e8c1f53fca4802873c2d1d650dc7b` | `45e576c03fbd213822fc5d6385e305198b94e191ffe12970e1da0479e51ea061` |
| `src/WebServices/MobileService/Module.bsl` | `d2c132b0adb4a474f48573ff9db13c18d3be90cf11b95c073441a955bb00d4e5` | `55ee10ec43f5f20124a508928d5228e7d2ae79676423e39817dd01bf593fa166` |
| `src/WebServices/SiteExchange2/SiteExchange2.mdo` | `e0f74bceca89c3ab09230ed4bdf04628c35c4c8b5b2e62144bf4f7f10f781909` | `de915c4e8ce7c5cfb34c34cd480d8a0df172313c0e2c7ecacc052b09f9ec35fa` |
| `src/WebServices/SiteExchange2/Module.bsl` | `8d2cbe724e3e3b9f88c3e8e9e13f499b7ad99bfc3b50245e6b57d1ea06d7d5a4` | `fcebd7fd434d1d4432f557e212c28700b2e690ea8c625390634cf77a1f752405` |
| `src/XDTOPackages/EquipmentService_2_0_0_3/EquipmentService_2_0_0_3.mdo` | `5f674d844074d01f24d9d51965304310ba36332737547bbd0d59a154cf8ef50a` | `5e5484e345a5aab1917597a0d7132aae57cf86274f87cd7c47487be62f3e754e` |
| `src/XDTOPackages/EquipmentService_2_0_0_3/Package.xdto` | `3861458625890d8b08396a94571ac7e720016671878839bb079244deaa787bcd` | `289b85d18b7af9d2110c2458b9ec4338218954c0aeac6a45caa528bd20762477` |
| `src/XDTOPackages/EquipmentService/EquipmentService.mdo` | `0a4381fc1813a11a6fbf3670437df01018d57d565645bf7d8159eea12de298a5` | `d1c34b3fd319a5f86989bf0446f550bed251be615b618a8748dc1491cc09a63f` |
| `src/XDTOPackages/EquipmentService/Package.xdto` | `29b637c7c4b2ff7b32018e491d8acb10c4161af8cc0a371519e9685841650e42` | `c3b335098ead3caf8df89d9ee2f8bd9aaedc6a96d5b4e9800c90af3002217b82` |
| `src/XDTOPackages/EquipmentService_1_0_0_7/EquipmentService_1_0_0_7.mdo` | `36e2e303ffe3797f4f0f9169aff4189fbeff2fa54fc8a398d314f02b0852d21b` | `b2ffb0d3ae1b3c833008ca8c8f6007e519c8bd599768076143a56d82c2cf4386` |
| `src/XDTOPackages/EquipmentService_1_0_0_7/Package.xdto` | `c9572b48b39828648ad9071ec76887293d09a52e6067638161e8c110e0cbf111` | `4e65498250d8ed8d755f2336217998b71273a42f2c98b8df9897fcec9520dd0e` |
| `src/XDTOPackages/EquipmentService_1_0_0_6/EquipmentService_1_0_0_6.mdo` | `4a66f7e0d5a4e96c1f6a607f3411764ca7811f5a9083095f6951ab861fc4851b` | `821fbdefa50ab348c71bc9e29240d187456e787ac845157832d1f2146065a817` |
| `src/XDTOPackages/EquipmentService_1_0_0_6/Package.xdto` | `851532b51367301c122aef6e7ccf9b7a358fb6f573d243e214a13da207267331` | `68e26c6c1eb405d4f307c2cfd82dd9e34b7bf64cc576e6ddce5ddb222f092293` |
| `src/XDTOPackages/MobileClientIntegration/MobileClientIntegration.mdo` | `f08a3d5db897e59502f3d016fa41e2f071d1f2cd1a0b3fb48c8f6c82686e9857` | `e56f88b11947eea51436ff17b3ad473294dc399670993d3d73be1aad7d5a8c6c` |
| `src/XDTOPackages/MobileClientIntegration/Package.xdto` | `cc9d3b0f6d3b06e589056531ddfa685396839806d5332a938048a1d9bb693a8e` | `cee212f62c2a40d439b084bacc7ee5910c8862d1dd5cf4b6ee992e49630227ac` |
| `src/XDTOPackages/CommerceML205a/CommerceML205a.mdo` | `584f882a9101a450ca7988390f1de07d01d1f5f2a94f8e73965ef56f94bc9105` | `9b57e56a80389c06b068b06ad33a36fad6ea8cb9557bd50eab229a7acb036c3f` |
| `src/XDTOPackages/CommerceML205a/Package.xdto` | `e7120d4e877b4e61bc7dd706db7a24379f165b3208f4c31f6b09dacc8d2b70ff` | `98633eab2b469c22fa2a6eafe8dd83b5e2dd675bc9eb8151515543ecd94e7644` |
| `src/XDTOPackages/CommerceML210/CommerceML210.mdo` | `0e491bba44dd9e50c34f785c95c557deff7c84d1d5b1a7af43af21e45bf747b6` | `a0e3354fa2911daccc67e1d67609d1f4626ce64122699ac98f9e73cbdf92c2bd` |
| `src/XDTOPackages/CommerceML210/Package.xdto` | `6c89dc9d415da175e6b30815610779e6e6df8c8b455d7fc20e6dad849d643da7` | `57ac03842abe942f17c90b409a9a1955fab9dc94f3e9ed68a96c3df63d371905` |

The ignored Retail corpus is never required by CI. Its source paths are the
reduced artifact paths above relative to
`Retail_edt_project/Розница_базовая/`.
