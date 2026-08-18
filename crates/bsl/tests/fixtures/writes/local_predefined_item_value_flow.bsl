Procedure AddPredefined_Yoda(Parameters = Undefined) Export

    If Parameters = Undefined Then
        Parameters = InfobaseUpdateDNSWE.NewUpdateParameters();
    EndIf;

    MethodName = InfobaseUpdateDNSWE.TempletMethodNameCreatingPredefined("Yoda", Metadata.ChartsOfCharacteristicTypes.DetailTypesOfNamedProducts);

    InfobaseUpdateDNSWE.LogHendlerStart(MethodName);

	#Region elements_FaultyProductDetails

	PredefinedItem = DiscountedProduct.GetObject();

	NStrDescription = "en = 'Discounted product'; ru = 'Уцененный товар'";

	NationalLanguageSupportServer.FillAttribute(PredefinedItem, "Description", NStrDescription);

    PredefinedItem.Write();

	#EndRegion
