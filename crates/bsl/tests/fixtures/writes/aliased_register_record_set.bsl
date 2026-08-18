Procedure CreateOrderExchangesRecords(Cancel)

	ExchangesSumAndRelatedDocument = Retail.DataForPaymentOrderDocuments(OrderPayments, DocumentBasis);
	RelatedDocument = ExchangesSumAndRelatedDocument.RelatedDocument;
	ExchangesSum = ExchangesSumAndRelatedDocument.ExchangesSum;

	OrderExchangesRecords = RegisterRecords.OrderExchanges;
	OrderExchangesRecords.Clear();
	OrderExchangesRecords.Write = True;

	If Not ValueIsFilled(ExchangesSum) Then
		Return;
	EndIf;

	If Not ValueIsFilled(RelatedDocument) Then
		PreMessage = NStr("ru = 'Обмен не возможен. %1 не связана с новым Заказом клиента.';
							  |en = 'Exchange is not possible. %1 is not connected to the new Customer Order.'");
		Message = StrTemplate(PreMessage, DocumentBasis);
		Common.MessageToUser(Message);

		Cancel = True;
		Return;
	EndIf;

	NewRecord = OrderExchangesRecords.Add();
	NewRecord.RecordType = AccumulationRecordType.Receipt;
	NewRecord.Recorder = Ref;
	NewRecord.Active = True;
	NewRecord.Period = Date;
	NewRecord.Request = RelatedDocument;
	NewRecord.Amount = ExchangesSum;

	SetPrivilegedMode(True);
	OrderExchangesRecords.Write();
	SetPrivilegedMode(False);

EndProcedure
