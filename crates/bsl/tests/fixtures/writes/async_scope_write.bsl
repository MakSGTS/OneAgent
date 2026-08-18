&AtClient
Async Procedure SaveCurrentFile()
	CurrentData = Items.Content.CurrentData;

	If CurrentData = Undefined Then
		Return;
	EndIf;

	Dialogue = New FileDialog(FileDialogMode.Save);
	Dialogue.Title = NStr("ru='Сохранить файл как'; en='Save file as'");
	Dialogue.FullFileName = CurrentData.FileName;

	If ValueIsFilled(CurrentData.FileExtension) Then
		Dialogue.Filter = StrTemplate(NStr("ru = 'Текущий файл (*%1*)|*%1*'; en = 'Current file (*%1*)|*%1*'"),
			CurrentData.FileExtension);
	EndIf;
	Dialogue.Filter = Dialogue.Filter +  NStr("ru = '|Все Файлы (*.*)|*.*'; en = '|All files (*.*)|*.*'");
	FilePaths = Await Dialogue.ChooseAsync();
	If FilePaths = Undefined Or FilePaths.Count() = 0 Then
		Return;
	EndIf;

	FilePath = FilePaths[0];

	If ValueIsFilled(CurrentData.FileExtension) Then
		ExtensionPosition = StrFind(FilePath, CurrentData.FileExtension, SearchDirection.FromEnd);
		If ExtensionPosition = 0 Then
			FilePath = FilePath + CurrentData.FileExtension;
		EndIf;
	EndIf;

	Try
		BinaryData = GetFromTempStorage(CurrentData.FileData);
		BinaryData.Write(FilePath);
	Except
		Raise NStr("ru='Ошибка при сохранении файла:'; en='Error while saving file:'") + ErrorDescription();
	EndTry;
EndProcedure
