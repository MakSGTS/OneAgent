Procedure CommandProcessingCompletion(QuestionResult, AdditionalParameters) Export

	Form = AdditionalParameters.Form; // ManagedFormExtensionForCatalogs -
	ReportingSet = AdditionalParameters.ReportingSet;
	ReportKind = AdditionalParameters.ReportKind;
	Response = QuestionResult;

	If Response = DialogReturnCode.Yes Then
		Form.Write();
	ElsIf Response = DialogReturnCode.No Then
		Return;
	EndIf;
