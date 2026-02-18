After Effects Rust プラグイン 動的UI Tips
1. AE と Premiere の根本的な違い（最重要）
AE SDK の PF_UpdateParamUI は、変更できるフラグが ホストによって異なります。
フラグ	After Effects	Premiere Pro
PF_PUI_DISABLED	✅ 有効	✅ 有効
PF_PUI_INVISIBLE	❌ 無効	✅ 有効
AE でパラメータを表示/非表示にするには AEGP Stream スイートを使う必要があります。
// AE: INVISIBLE は効かない。AEGP を使うlet aegp_eff = in_data.effect().aegp_effect(plugin_id)?;let stream = aegp_eff.new_stream_by_index(plugin_id, param_index as i32)?;stream.set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;// Premiere: INVISIBLE は有効let mut p = params.cloned();let mut pd = p.get_mut(PARAM_KEY)?;pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, hidden);pd.update_param_ui()?;
コードでは in_data.is_premiere() で分岐します。
2. AEGP を使うための準備
Plugin struct に PluginId を持たせ、GlobalSetup で登録します。
#[derive(Default)]struct Plugin {    my_id: ae::aegp::PluginId, // ae_sys::A_long (= i32) なので Default = 0}// GlobalSetup 内if let Ok(suite) = ae::aegp::suites::Utility::new()    && let Ok(id) = suite.register_with_aegp("AOD_PluginName"){    self.my_id = id;}
PluginId は i32 の型エイリアスなので #[derive(Default)] がそのまま使えます。
3. params.cloned() + get_mut() パターンが必須
UpdateParamsUi に渡される params は AE の内部メモリを指すポインタで、一部は書き込み禁止です。set_ui_flag + update_param_ui を呼ぶには必ずクローンを作ります。
// NG: params を直接触ろうとすると読み取り専用で失敗する場合がある// OK: cloned() でコピーを作ってから操作するlet mut p = params.cloned();{    let mut pd = p.get_mut(SOME_PARAM)?;    pd.set_ui_flag(ae::ParamUIFlags::DISABLED, true);    pd.update_param_ui()?;  // これを忘れると AE に通知されない}
update_param_ui() の呼び忘れはサイレントに失敗するので注意。
4. params.index(key) で AEGP 用のインデックスを取得
AEGP の new_stream_by_index に渡すインデックスは、params.index(ParamKey) から取得します。params_setup で登録された順序と一致します。
let idx = params.index(Params::TargetColor0)    .ok_or(ae::Error::InvalidIndex)? as i32;aegp_eff.new_stream_by_index(plugin_id, idx)?    .set_dynamic_stream_flag(ae::aegp::DynamicStreamFlags::Hidden, false, hidden)?;
5. 動的UI が機能するための前提条件チェックリスト
以下がすべて揃っていないと動的UIは動きません。
[ ] build.rs の PiPL に OutFlags::SendUpdateParamsUi を設定[ ] GlobalSetup で out_data.set_out_flag(OutFlags::SendUpdateParamsUi, true)[ ] トリガーとなるパラメータに ParamFlag::SUPERVISE を付与    → これがないと値変更時に UpdateParamsUi が発火しない[ ] UpdateParamsUi ハンドラで実際に処理を実装[ ] update_param_ui() を各パラメータごとに呼び出す（DISABLED 変更時）[ ] AE での表示/非表示は AEGP Stream を使う（INVISIBLE は不可）
6. ParamFlag::SUPERVISE が必要なパラメータ
「この値が変わったら UI を更新してほしい」 というパラメータに付けます。
params.add_with_flags(    Params::ExtractionCount,    "Extraction Count",    PopupDef::setup(|d| { ... }),    ParamFlag::SUPERVISE,  // ← これがないと UpdateParamsUi が発火しない    ParamUIFlags::NONE,)?;
また、グレーアウト制御のための Enable チェックボックスにも SUPERVISE が必要です。
7. スライダーが表示時に勝手に開く問題
INVISIBLE を解除してスライダーを再表示すると、AE の仕様でスライダーが展開してしまうことがあります。START_COLLAPSED を毎回セットすることで防げます。
let mut pd = p.get_mut(SOME_SLIDER)?;pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);pd.set_flag(ae::ParamFlag::START_COLLAPSED, true); // 毎回セットで展開を防ぐpd.update_param_ui()?;
8. ネストしたグループ（Topic）は AE で不安定
add_group によるネストは AE 上でグループ名の消失・開閉挙動の不具合が起きる場合があります。フラットなリスト + INVISIBLE/DISABLED の組み合わせの方が安定しています。
9. AEGP の set_dynamic_stream_flag の引数の意味
stream.set_dynamic_stream_flag(    ae::aegp::DynamicStreamFlags::Hidden,    false,   // undoable: 通常 false    hidden,  // enabled: true = Hidden フラグ有効 = 非表示)?;
enabled: true が「フラグを有効化（= 非表示にする）」という意味です。直感と逆に感じる場合は !visible で渡すと読みやすくなります。
10. ホスト分岐のパターン（まとめ）
if in_data.is_premiere() {    // Premiere: INVISIBLE で非表示    let mut p = params.cloned();    let mut pd = p.get_mut(KEY)?;    pd.set_ui_flag(ae::ParamUIFlags::INVISIBLE, !vis);    pd.update_param_ui()?;} else {    // AE: AEGP Stream で非表示    aegp_eff.new_stream_by_index(plugin_id, idx)?        .set_dynamic_stream_flag(Hidden, false, !vis)?;}// DISABLED はどちらも同じlet mut p = params.cloned();let mut pd = p.get_mut(KEY)?;pd.set_ui_flag(ae::ParamUIFlags::DISABLED, disabled);pd.update_param_ui()?;
この構造が after-effects-rs の supervisor サンプル（examples/supervisor/）でも採用されているパターンです。実装に迷ったらそちらも参照してください。
