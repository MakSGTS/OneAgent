#Region Internal

#Region SubsystemFilling

Procedure FillSecurityCollection(SecurityCollection) Export

EndProcedure

Procedure ExerciseSecurityCollection(SecurityCollection)

	FillSecurityCollection(SecurityCollection);
	DynamicSecurityOverridable.FillSecurityCollection(SecurityCollection);

EndProcedure

#EndRegion

#EndRegion
