"""Pipeline configuration and field mapping module.

Provides tools for configuring analysis pipelines and mapping custom dataset fields
to standardized field names required by various analyses.
"""
from dataclasses import dataclass, field
from typing import Dict, List, Optional, Set, Any
import pandas as pd
from pathlib import Path
import yaml
import json

@dataclass
class FieldMapping:
    """Maps custom dataset fields to standardized field names."""
    source_field: str
    target_field: str
    transform: Optional[str] = None
    validation: Optional[str] = None

@dataclass
class AnalysisConfig:
    """Configuration for a specific type of analysis."""
    analysis_type: str
    required_fields: Set[str]
    optional_fields: Set[str]
    field_mappings: Dict[str, FieldMapping] = field(default_factory=dict)
    
    def validate(self) -> List[str]:
        """Validate the configuration.
        
        Returns:
            List of validation error messages
        """
        errors = []
        mapped_fields = set(self.field_mappings.keys())
        
        # Check required fields
        missing_required = self.required_fields - mapped_fields
        if missing_required:
            errors.append(f"Missing required fields: {missing_required}")
            
        return errors

class PipelineConfig:
    """Manages pipeline configuration and field mapping."""
    
    # Standard analysis types and their required/optional fields
    ANALYSIS_TYPES = {
        "survival": {
            "required": {"time", "event", "group"},
            "optional": {"age", "sex", "treatment", "competing_risk", "left_truncation", "cluster"}
        },
        "propensity": {
            "required": {"treatment", "outcome"},
            "optional": {"age", "sex", "comorbidity", "socioeconomic", "healthcare_access", "region"}
        },
        "diagnostic": {
            "required": {"test_result", "true_condition"},
            "optional": {"test_date", "severity", "test_batch", "lab_id", "specimen_type"}
        },
        "time_series": {
            "required": {"date", "count", "location"},
            "optional": {"population", "intervention_date", "variant", "vaccination_rate", "testing_rate"}
        },
        "genetic_epi": {
            "required": {"variant", "phenotype", "population"},
            "optional": {"age", "sex", "ancestry", "gene_region", "allele_freq"}
        },
        "environmental": {
            "required": {"exposure", "outcome", "location"},
            "optional": {"time_period", "temperature", "pollution_level", "precipitation", "altitude"}
        },
        "transmission": {
            "required": {"case_id", "contact_id", "date"},
            "optional": {"setting", "duration", "distance", "mask_use", "ventilation", "variant"}
        },
        "vaccine_effectiveness": {
            "required": {"vaccination_status", "outcome", "time_since_vaccination"},
            "optional": {"vaccine_type", "dose_number", "age", "risk_group", "variant"}
        },
        "health_inequalities": {
            "required": {"outcome", "socioeconomic_status", "location"},
            "optional": {"education", "income", "healthcare_access", "race_ethnicity", "urban_rural"}
        },
        "outbreak_detection": {
            "required": {"date", "count", "location"},
            "optional": {"baseline", "threshold", "seasonality", "population_size", "reporting_delay"}
        }
    }
    
    def __init__(self, analysis_type: str):
        """Initialize pipeline configuration.
        
        Args:
            analysis_type: Type of analysis to configure
        """
        if analysis_type not in self.ANALYSIS_TYPES:
            raise ValueError(f"Unknown analysis type: {analysis_type}")
            
        self.analysis_type = analysis_type
        self.config = AnalysisConfig(
            analysis_type=analysis_type,
            required_fields=self.ANALYSIS_TYPES[analysis_type]["required"],
            optional_fields=self.ANALYSIS_TYPES[analysis_type]["optional"]
        )
    
    def suggest_mappings(self, df: pd.DataFrame) -> Dict[str, List[str]]:
        """Suggest field mappings based on column names.
        
        Args:
            df: Input DataFrame
            
        Returns:
            Dict mapping required fields to suggested source fields
        """
        suggestions = {}
        columns = set(df.columns)
        
        # Common variations of field names
        field_variants = {
            "time": ["time", "duration", "followup", "follow_up", "period"],
            "event": ["event", "outcome", "death", "failure", "status"],
            "group": ["group", "treatment", "arm", "cohort"],
            "age": ["age", "age_years", "age_at_baseline"],
            "sex": ["sex", "gender"],
            "treatment": ["treatment", "intervention", "drug", "therapy"]
        }
        
        for required in self.config.required_fields:
            matches = []
            if required in field_variants:
                for variant in field_variants[required]:
                    matches.extend([col for col in columns if variant.lower() in col.lower()])
            suggestions[required] = sorted(set(matches))
            
        return suggestions
    
    def map_field(self, target_field: str, source_field: str, 
                  transform: Optional[str] = None,
                  validation: Optional[str] = None) -> None:
        """Map a source field to a target field.
        
        Args:
            target_field: Standardized field name
            source_field: Original field name in dataset
            transform: Optional transformation to apply
            validation: Optional validation rule
        """
        if target_field not in (self.config.required_fields | self.config.optional_fields):
            raise ValueError(f"Unknown target field: {target_field}")
            
        self.config.field_mappings[target_field] = FieldMapping(
            source_field=source_field,
            target_field=target_field,
            transform=transform,
            validation=validation
        )
    
    def validate_mapping(self, df: pd.DataFrame) -> List[str]:
        """Validate field mappings against a DataFrame.
        
        Args:
            df: DataFrame to validate against
            
        Returns:
            List of validation error messages
        """
        errors = []
        
        # Check basic configuration
        errors.extend(self.config.validate())
        
        # Check source fields exist
        for mapping in self.config.field_mappings.values():
            if mapping.source_field not in df.columns:
                errors.append(f"Source field not found: {mapping.source_field}")
                
        return errors
    
    def transform_data(self, df: pd.DataFrame) -> pd.DataFrame:
        """Apply field mappings and transformations to DataFrame.
        
        Args:
            df: Input DataFrame
            
        Returns:
            Transformed DataFrame with standardized fields
        """
        result = df.copy()
        
        for mapping in self.config.field_mappings.values():
            # Rename field
            if mapping.source_field != mapping.target_field:
                result = result.rename(columns={
                    mapping.source_field: mapping.target_field
                })
            
            # Apply transformation if specified
            if mapping.transform:
                # Safe eval of simple transformations
                if mapping.transform.startswith('lambda'):
                    transform_func = eval(mapping.transform)
                    result[mapping.target_field] = result[mapping.target_field].apply(transform_func)
                    
        return result
    
    def save_config(self, path: str) -> None:
        """Save configuration to file.
        
        Args:
            path: Path to save configuration
        """
        config = {
            "analysis_type": self.analysis_type,
            "field_mappings": {
                target: {
                    "source_field": mapping.source_field,
                    "transform": mapping.transform,
                    "validation": mapping.validation
                }
                for target, mapping in self.config.field_mappings.items()
            }
        }
        
        with open(path, 'w') as f:
            if path.endswith('.json'):
                json.dump(config, f, indent=2)
            else:
                yaml.dump(config, f)
    
    @classmethod
    def load_config(cls, path: str) -> 'PipelineConfig':
        """Load configuration from file.
        
        Args:
            path: Path to configuration file
            
        Returns:
            PipelineConfig instance
        """
        with open(path) as f:
            if path.endswith('.json'):
                config = json.load(f)
            else:
                config = yaml.safe_load(f)
                
        instance = cls(config['analysis_type'])
        for target, mapping in config['field_mappings'].items():
            instance.map_field(
                target_field=target,
                source_field=mapping['source_field'],
                transform=mapping.get('transform'),
                validation=mapping.get('validation')
            )
            
        return instance 