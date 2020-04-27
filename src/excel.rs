use crate::db::db_models::DocPerson;
use calamine::{Reader, open_workbook, Xlsx, Error, RangeDeserializerBuilder, RangeDeserializer};


#[derive(Debug)]
pub enum ExcelError {
    Error(String)
}




impl<'a> std::convert::From<calamine::Error> for ExcelError {
    fn from(e:calamine::Error) -> Self {
        ExcelError::Error(format!("{:?}",e))
    }
}

impl<'a> std::convert::From<calamine::XlsxError> for ExcelError {
    fn from(e:calamine::XlsxError) -> Self {
        ExcelError::Error(format!("{:?}",e))
    }
}

impl<'a> std::convert::From<calamine::DeError> for ExcelError {
    fn from(e:calamine::DeError) -> Self {
        ExcelError::Error(format!("{:?}",e))
    }
}


//Read the data from file
pub fn read<'a>(path:String) -> Result<(Vec<DocPerson>, Vec<String>),ExcelError> {
    let mut workbook:Xlsx<_> = open_workbook(path)?;

    //Collection to store person nodes
    let mut persons:Vec<DocPerson> = Vec::new();
    
    //Collection to store error/waring about rows while reading
    let mut warnings:Vec<String> = Vec::new();

    //Extract out the sheets
    let sheet_count = workbook.sheet_names().len();

    for sheet_index in 0..sheet_count {
        let range = workbook.worksheet_range_at(sheet_index).ok_or(Error::Msg("cannot find sheet"))??;

        let row_iter:RangeDeserializer<'_, calamine::DataType, (
            String, // QID
            String, // Timestamp
            String, // Name
            String, // Email
            String, // Phone
            String, // City
            String, // Gender
            u8,     // Age
            String, // Education
            String, // Profession
            String, // Verbal Ability
            String, // Seeking
        )> = RangeDeserializerBuilder::new().from_range(&range)?;
        
        //We Got some data  
        for (index, row) in row_iter.enumerate() {
            if let Ok((
                qid, 
                _,
                name, 
                email, 
                phone, 
                city,
                gender,
                age,
                education,
                profession, 
                verbal_ability, 
                seeking)) = row {
              //TODO : Validate data
    
                
                // Check for valid qid. TODO : Pattern matching 'Q-{1..}'
                if qid.contains("Q-") {
                    persons.push(DocPerson {
                        qid:qid,
                        name:name,
                        email:email,
                        phone:phone,
                        profession:profession,
                        age:age,
                        gender:gender,
                        response_rating:0,
                        city:city,
                        seeking:seeking,
                        verbal_ability:verbal_ability,
                        education:education,
                        languages:vec![String::from("English")],
                    });
                }else{
                    warnings.push(format!("Invalid qid in sheet {} row {}", sheet_index + 1, index));
                }
    
            }else{
                //Print Invalid Row
                warnings.push(format!("Invalid data in sheet {} row {}", sheet_index + 1, index));
            }
            
        }
    }



    if persons.len() > 0 || warnings.len() > 0 {
        return Ok((persons, warnings))
    }

    Err(ExcelError::Error(String::from("Sheet is empty")))

}