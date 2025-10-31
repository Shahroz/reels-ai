use actix_web::dev::{ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::{web, App, Error};
use awc::Client;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::api::app::{common_api_routes, init_app_config, init_node_context, init_node_registry};
use crate::api::jwt_middleware::JwtMiddleware;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkflowId {
    pub workflow_id: Uuid
}

pub async fn create_test_app() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response=ServiceResponse,
        Config=(),
        InitError=(),
        Error=Error,
    >
> {
    _ = dotenv();
    let reqwest_client = reqwest::Client::default();
    let app_config = init_app_config();
    let node_registry = init_node_registry(&app_config).await;
    let node_context = init_node_context(node_registry.clone(), &app_config);
    App::new()
        .app_data(web::Data::new(Client::default()))
        .app_data(web::Data::new(reqwest_client.clone()))
        .app_data(web::Data::new(app_config.clone()))
        .app_data(web::Data::new(node_context.clone()))
        .service(
            web::scope("/api/jwt")
                .wrap(JwtMiddleware)
                .configure(common_api_routes))
        .service(
            web::scope("/api/v1")
                .configure(common_api_routes))
}

#[cfg(FALSE)]
mod tests {
    use serde_json::json;
    use super::*;
    use actix_web::{test, App, Error};
    use actix_web::body::{BoxBody, MessageBody};
    use actix_web::web;
    use crate::api::app::{common_api_routes, init_node_context};
    use awc::Client;
    use dotenvy::dotenv;
    use crate::api::app::init_app_config;
    use crate::api::app::init_node_registry;
    use actix_web::http::StatusCode;
    use actix_web::dev::{Service, ServiceFactory, ServiceRequest, ServiceResponse};
    use reqwest::Request;
    use crate::api::jwt_middleware::{Claims, JwtMiddleware};
    use actix_web::http::header;
    use uuid::Uuid;
    use crate::api::views::workflow::{CreateWorkflowFromTemplateRequest, WorkflowExecutorRunArguments};
    use crate::nodes::prelude::*;
    use url::Url;
    use std::str::FromStr;
    use schemars::_private::NoSerialize;
    use serde::Deserialize;
    use serde_with::serde_derive::Serialize;
    use gennodes_datatypes::use_case::CompanyUseCaseV3;
    use gennodes_datatypes::use_case::CompanyUseCase;
    use gennodes_common::traits::llm_traits::FewShotsOutput;
    use crate::api::views::workflow::workflow_edit::EditWorkflowRequest;
    use gennodes_common::ui::TableResult;
    use crate::nodes::workflow_templates::icp_unified_search::SearchCompaniesUnifiedParameters;

    #[actix_rt::test]
    async fn test_db_health() {
        let app = create_test_app().await;
        let service = test::init_service(app).await;
        let req = test::TestRequest::get().uri("/api/v1/db_health").to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body();
        assert_eq!(status, StatusCode::OK);
    }

    #[actix_rt::test]
    async fn test_jwt_missing_tokens() {
        let app = create_test_app().await;
        let service = test::init_service(app).await;
        let req = test::TestRequest::get().uri("/api/jwt/db_health").to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[actix_rt::test]
    async fn test_jwt_tokens_verified() {
        let app = create_test_app().await;
        let service = test::init_service(app).await;
        let claims = Claims::new_with_tenant_id("123");
        let jwt_token = claims.to_token().unwrap();
        log::debug!("{:?}", jwt_token);
        let req = test::TestRequest::get().uri("/api/jwt/db_health")
            .append_header((header::AUTHORIZATION, jwt_token))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body();
        assert_eq!(status, StatusCode::OK);
    }

    #[actix_rt::test]
    async fn test_jwt_workflow_creation() {
        let app = create_test_app().await;
        let service = test::init_service(app).await;
        let tenant_id = Uuid::new_v4();
        let claims = Claims::new_with_tenant_id(&tenant_id.to_string());
        let jwt_token = claims.to_token().unwrap();

        // create a workflow from a template
        let workflow_template = CreateWorkflowFromTemplateRequest {
            template_parameters: WorkflowTemplateParameters::ReferenceFromURL(
                ReferenceFromURL{
                    url: Url::from_str("https://bounti.ai").unwrap()
                }
            ),
            ..Default::default()
        };
        let req = test::TestRequest::post().uri("/api/jwt/workflow-from-template")
            .set_json(workflow_template.clone())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        let json_value = serde_json::from_slice(&body).unwrap();
        let workflow: WorkflowId = serde_json::from_value(json_value).unwrap();
        let workflow_id = workflow.workflow_id.to_string();
        assert_eq!(status, StatusCode::OK);

        // run the workflow
        let workflow_run = WorkflowExecutorRunArguments{
            workflow_id: workflow.workflow_id,
            action: WorkflowExecutorAction::Pull {
                node_name: Some("reference".to_string()),
                n_items: 1
            },
            simple_output: None,
            with_sources: None,
            tenant_id: None
        };
        let req = test::TestRequest::post().uri("/api/jwt/workflow/run")
            .set_json(workflow_run.clone())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        assert_eq!(status, StatusCode::OK);

        // get the results by workflow_id
        let req = test::TestRequest::get().uri(format!("/api/jwt/nodes/results/by_content_type/Reference?workflow_id={workflow_id}").as_str())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        let table_results_workflow_id: TableResult = serde_json::from_slice(&body).unwrap();
        assert_eq!(status, StatusCode::OK);

        // get the results without workflow_id (implicit tenant_id is used)
        // the results from using a workflow_id and without in this case should be the same
        let req = test::TestRequest::get().uri(format!("/api/jwt/nodes/results/by_content_type/Reference").as_str())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        let table_results_wo_workflow_id: TableResult = serde_json::from_slice(&body).unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(table_results_workflow_id, table_results_wo_workflow_id);
    }

    #[actix_rt::test]
    async fn test_update_name_description() {
        let app = create_test_app().await;
        let service = test::init_service(app).await;
        let tenant_id = Uuid::new_v4();
        let claims = Claims::new_with_tenant_id(&tenant_id.to_string());
        let jwt_token = claims.to_token().unwrap();

        // create a workflow from a template
        let workflow_template = CreateWorkflowFromTemplateRequest {
            template_parameters: WorkflowTemplateParameters::ReferenceFromURL(
                ReferenceFromURL{
                    url: Url::from_str("https://bounti.ai").unwrap()
                }
            ),
            ..Default::default()
        };
        let req = test::TestRequest::post().uri("/api/jwt/workflow-from-template")
            .set_json(workflow_template.clone())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        let json_value = serde_json::from_slice(&body).unwrap();
        let workflow: WorkflowId = serde_json::from_value(json_value).unwrap();

        // update name and description
        let edit_workflow_request = EditWorkflowRequest{
            id: workflow.workflow_id,
            name: Some("New name".to_string()),
            description: Some("New description".to_string()),
        };
        let req = test::TestRequest::post().uri("/api/jwt/workflow/edit")
            .set_json(edit_workflow_request)
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        assert_eq!(status, StatusCode::OK);

        // load the state
        let req = test::TestRequest::get().uri(format!("/api/jwt/workflow/{}/details", workflow.workflow_id.to_string()).as_str())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        let workflow_state: WorkflowExecutorNodeState = serde_json::from_slice(&body).unwrap();
        assert_eq!(status, StatusCode::OK);
        assert_eq!(workflow_state.name.unwrap(), "New name");
        assert_eq!(workflow_state.description.unwrap(), "New description");
    }

    #[actix_rt::test]
    async fn test_pitch_1() {
        test_jwt_workflow_creation_pitch(include_str!("workflows/pitch_workflow_1.json")).await;
    }

    async fn test_jwt_workflow_creation_pitch(pitch_workflow_raw_string: &str) {
        let app = create_test_app().await;
        let service = test::init_service(app).await;
        let tenant_id = Uuid::new_v4();
        let claims = Claims::new_with_tenant_id(&tenant_id.to_string());
        let jwt_token = claims.to_token().unwrap();

        let template_parameters_raw_json = serde_json::Value::from_str(pitch_workflow_raw_string).unwrap();

        let template_parameters: WorkflowTemplateParameters = serde_json::from_value(template_parameters_raw_json).unwrap();

        // create a workflow from a template
        let workflow_template = CreateWorkflowFromTemplateRequest {
            template_parameters,
            ..Default::default()
        };
        let req = test::TestRequest::post().uri("/api/jwt/workflow-from-template")
            .set_json(workflow_template.clone())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        let json_value = serde_json::from_slice(&body).unwrap();
        log::debug!("{}", json_value);
        let workflow: WorkflowId = serde_json::from_value(json_value).unwrap();
        assert_eq!(status, StatusCode::OK);

        // run the workflow 5 times
        for _ in 0..2 {
            let workflow_run = WorkflowExecutorRunArguments {
                workflow_id: workflow.workflow_id,
                action: WorkflowExecutorAction::Pull {
                    node_name: Some("map_to_pitch".to_string()),
                    n_items: 1
                },
                simple_output: None,
                with_sources: None,
                tenant_id: None
            };
            let req = test::TestRequest::post().uri("/api/jwt/workflow/run")
                .set_json(workflow_run.clone())
                .append_header((header::AUTHORIZATION, jwt_token.clone()))
                .to_request();
            let res = service.call(req).await.unwrap();
            let status = res.status();
            let body = res.into_body().try_into_bytes().unwrap();
            assert_eq!(status, StatusCode::OK);
        }
    }

    #[actix_rt::test]
    async fn test_jwt_workflow_creation_use_cases_onboarding() {
        let app = create_test_app().await;
        let service = test::init_service(app).await;
        let tenant_id = Uuid::new_v4();
        let claims = Claims::new_with_tenant_id(&tenant_id.to_string());
        let jwt_token = claims.to_token().unwrap();

        let template_parameters_raw_json = json!{{
            "UseCasesFromURL": {
                "url": "https://bounti.ai/"
            }
        }};

        let template_parameters: WorkflowTemplateParameters = serde_json::from_value(template_parameters_raw_json).unwrap();

        // create a workflow from a template
        let workflow_template = CreateWorkflowFromTemplateRequest {
            template_parameters,
            ..Default::default()
        };
        let req = test::TestRequest::post().uri("/api/jwt/workflow-from-template")
            .set_json(workflow_template.clone())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        let json_value = serde_json::from_slice(&body).unwrap();
        let workflow: WorkflowId = serde_json::from_value(json_value).unwrap();
        assert_eq!(status, StatusCode::OK);

        // run the workflow 5 times
        let workflow_run = WorkflowExecutorRunArguments {
            workflow_id: workflow.workflow_id,
            action: WorkflowExecutorAction::Pull {
                node_name: Some("use_case".to_string()),
                n_items: 1
            },
            simple_output: Some(true),
            with_sources: None,
            tenant_id: None
        };
        let req = test::TestRequest::post().uri("/api/jwt/workflow/run")
            .set_json(workflow_run.clone())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        assert_eq!(status, StatusCode::OK);
    }


    #[actix_rt::test]
    async fn test_jwt_workflow_creation_use_cases_discovery() {
        let app = create_test_app().await;
        let service = test::init_service(app).await;
        let tenant_id = Uuid::new_v4();
        let claims = Claims::new_with_tenant_id(&tenant_id.to_string());
        let jwt_token = claims.to_token().unwrap();

        let template_parameters_raw_json = json!{{
            "AutoprospectingUseCasesDiscovery": {
                "domain": "https://bounti.ai/",
                "name": "Bounti"
            }
        }};

        let template_parameters: WorkflowTemplateParameters = serde_json::from_value(template_parameters_raw_json).unwrap();

        // create a workflow from a template
        let workflow_template = CreateWorkflowFromTemplateRequest {
            template_parameters,
            ..Default::default()
        };
        let req = test::TestRequest::post().uri("/api/jwt/workflow-from-template")
            .set_json(workflow_template.clone())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        let json_value = serde_json::from_slice(&body).unwrap();
        let workflow: WorkflowId = serde_json::from_value(json_value).unwrap();
        assert_eq!(status, StatusCode::OK);

        // run the workflow 5 times
        let workflow_run = WorkflowExecutorRunArguments {
            workflow_id: workflow.workflow_id,
            action: WorkflowExecutorAction::Pull {
                node_name: Some("use_cases_scraped_mapped".to_string()),
                n_items: 1
            },
            simple_output: Some(true),
            with_sources: None,
            tenant_id: None
        };
        let req = test::TestRequest::post().uri("/api/jwt/workflow/run")
            .set_json(workflow_run.clone())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        assert_eq!(status, StatusCode::OK);
    }

    #[actix_rt::test]
    async fn test_jwt_workflow_creation_icp() {
        let app = create_test_app().await;
        let service = test::init_service(app).await;
        let tenant_id = Uuid::new_v4();
        let claims = Claims::new_with_tenant_id(&tenant_id.to_string());
        let jwt_token = claims.to_token().unwrap();

        let parameters = SearchCompaniesUnifiedParameters{
            existing_customers: vec![Url::from_str("https://redis.com").unwrap(), Url::from_str("https://zoom.com").unwrap()],
            use_case_description: Some("AI-assisted sales prospecting".to_string()),
            // max_number_of_results: Some(100),
            // candidates_multiplicator: Some(2),
            // apply_llm_scoring: Some(false)
        };

        let template_parameters: WorkflowTemplateParameters = WorkflowTemplateParameters::SearchCompaniesUnified(parameters);

        // create a workflow from a template
        let workflow_template = CreateWorkflowFromTemplateRequest {
            template_parameters,
            ..Default::default()
        };
        let req = test::TestRequest::post().uri("/api/jwt/workflow-from-template")
            .set_json(workflow_template.clone())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        let json_value = serde_json::from_slice(&body).unwrap();
        let workflow: WorkflowId = serde_json::from_value(json_value).unwrap();
        assert_eq!(status, StatusCode::OK);

        // run the workflow 5 times
        for _ in 0..2 {
            let workflow_run = WorkflowExecutorRunArguments {
                workflow_id: workflow.workflow_id,
                action: WorkflowExecutorAction::Pull {
                    node_name: Some("map_to_company_profile".to_string()),
                    n_items: 10
                },
                simple_output: None,
                with_sources: None,
                tenant_id: None
            };
            let req = test::TestRequest::post().uri("/api/jwt/workflow/run")
                .set_json(workflow_run.clone())
                .append_header((header::AUTHORIZATION, jwt_token.clone()))
                .to_request();
            let res = service.call(req).await.unwrap();
            let status = res.status();
            let body = res.into_body().try_into_bytes().unwrap();
            assert_eq!(status, StatusCode::OK);
        }
    }

    #[actix_rt::test]
    async fn test_jwt_microsites() {
        let app = create_test_app().await;
        let service = test::init_service(app).await;
        let tenant_id = Uuid::new_v4();
        let claims = Claims::new_with_tenant_id(&tenant_id.to_string());
        let jwt_token = claims.to_token().unwrap();

        let parameters_json = json!{{
            "CreateMicrosite": {
                "input": "{\"pitch\": {\"id\": \"ffd1b1bb-1362-4703-9e82-25feab19930c\", \"created_at\": \"2025-01-13T18:47:03.090000\", \"updated_at\": \"2025-01-13T18:47:03.090000\", \"prospect_request_id\": \"92ffa98a-43df-4cf8-8d88-49ab56171911\", \"title\": \"Strengthening Event Services with Atlassian Solutions\", \"description\": \"Why?\\nCompuSystems, a leading provider of event registration and lead management services, faces challenges in managing Atlassian licenses, ensuring data security during cloud migrations, and implementing effective IT Service Management. These pain points threaten their ability to deliver seamless event experiences and meet client expectations.\\n\\nWhy Now?\\nHaving recently signed an 8-show contract with Access Intelligence for comprehensive registration services through 2026, CompuSystems needs to ensure efficient operations and data integrity. Addressing these challenges now will solidify their reputation and operational efficiency amidst growing market demands.\\n\\nWhy Us?\\nAvaratak Consulting specializes in optimizing Atlassian tools, offering tailored solutions for license management and cloud migration. Our proven track record includes enhancing data security with a 90% successful migration rate and improving ITSM efficiency by 30%. Partnering with us will empower CompuSystems to strengthen their service delivery, ensuring they meet the new contractual obligations with enhanced support and resilience.\", \"opportunity_id\": \"8271a41a-4af3-417d-8bcb-d033eb718c27\", \"use_case_id\": null, \"use_case_ids\": null, \"published\": true, \"reasoning\": \"This pitch effectively aligns with CompuSystems' pressing need for improved operational management as they embark on significant event service commitments. By emphasizing the critical nature of streamlined Atlassian solutions for license management and cloud transitions, the pitch positions Avaratak Consulting as a timely and highly relevant partner. It underscores the urgent requirement to address operational inefficiencies and data security, leveraging our expertise in ensuring robust IT service management and disaster recovery capabilities, thereby enhancing CompuSystems' value proposition in the event management industry.\", \"score\": 9, \"use_case_description\": null, \"use_case_link\": null, \"use_case_title\": null, \"opportunity_description\": null, \"references\": null}, \"persona_group\": {\"id\": \"ad559856-ae94-4688-8f56-82decf060dae\", \"seller_company_id\": \"751a88a2-8820-468c-bf56-a636a7adb9c8\", \"job_goals\": \"- Develop and implement technology strategies aligned with business objectives\\n- Ensure efficient operation of IT infrastructure\\n- Drive technological innovation and transformation.\", \"pains\": \"- High costs and challenges of technology implementation\\n- Keeping up with rapid technological advancements\\n- Ensuring continuous alignment of technology with business goals.\", \"gains\": \"- Cost-effective technology solutions and seamless implementation\\n- Enhanced operational efficiency and productivity\\n- Sustained alignment of technology and business objectives.\", \"responsibilities\": \"- Lead technology strategy development and implementation\\n- Oversee IT operations and infrastructure\\n- Drive innovation and transformation initiatives.\", \"success_criteria\": \"- Successful execution of technology strategies\\n- Improved efficiency and productivity metrics\\n- Continuous alignment of technology with business goals.\", \"group_name\": \"Technology and Information Executive Leaders\", \"job_titles\": [\"CTO\", \"Chief Technology Officer\", \"CIO\", \"Chief Information Officer\"]}, \"seller_info\": {\"seller_name\": \"Avaratak Consulting\", \"seller_domain\": \"avaratak.com\", \"seller_use_case\": {\"id\": \"255552b3-df54-4903-8cfe-e8dcd256ba25\", \"title\": \"Optimizing Atlassian Solutions for Businesses\", \"link\": \"https://avaratak.com/services-2\", \"description\": \"Comprehensive support for Atlassian tools, enhancing organizational efficiency and data security.\", \"pain\": \"- Managing Atlassian licenses can be complex and time-consuming.\\n- Cloud migration processes often lead to data security and compliance concerns.\\n- Organizations struggle with tailored IT Service Management aligning with their workflows.\\n- Without proper backing, projects may face downtime and data loss during emergencies.\", \"problem_statement\": \"As organizations increasingly adopt Atlassian products, they face challenges in efficiently managing licenses, ensuring data security during cloud migrations, and implementing effective IT Service Management (ITSM) solutions. Additionally, disjointed support can lead to significant downtime and impact project delivery, highlighting the need for specialized services that address these issues holistically.\", \"results\": \"- Streamlined management of Atlassian licenses, reducing procurement time by 40%.\\n- Enhanced data security and operational compliance during cloud transitions, with a 90% successful migration rate.\\n- Improved user satisfaction with ITSM solutions, leading to a 30% increase in service delivery efficiency.\\n- Comprehensive disaster recovery solutions that ensure 100% data recovery success during emergencies.\", \"solution\": \"Avaratak Consulting provides specialized services for managing Atlassian tools, including license procurement and optimization, cloud migration assistance, and bespoke ITSM implementations. Their offerings span disaster recovery planning, project management enhancements, and tailored integrations with third-party systems, ensuring seamless operations and data integrity.\", \"is_profile_visable\": true, \"company_profile_id\": \"751a88a2-8820-468c-bf56-a636a7adb9c8\"}, \"user\": {\"id\": \"ed53c377-c401-453c-9db8-b2edd827b1d2\", \"email\": \"abe.durrant@avaratak.com\", \"created_at\": \"2024-11-12T16:41:57.368000\", \"company_profile_id\": \"751a88a2-8820-468c-bf56-a636a7adb9c8\", \"on_waitlist\": false, \"name\": \"Abraham Durrant\", \"clearbit_id\": null, \"email_verified\": true, \"clearbit_profile\": null, \"last_refresh\": null, \"work_email\": true, \"internal_c_s_notes\": \"\", \"prospect_briefs_viewed\": [\"8d5a931b-1182-4bc1-854a-9e3186d1270e\", \"008b991d-6957-4193-9711-a9a7a7b323a1\", \"438a831c-6128-4591-8f2b-117a9df2c95c\", \"17e34d7d-c3a2-496b-ad48-5d746874f01b\", \"96ef1e24-c96b-463e-b416-93322db8aca6\", \"96ef1e24-c96b-463e-b416-93322db8aca6\", \"90c15090-4f16-4da9-a67d-d3f4ebff8f01\", \"8e701b53-5bbb-4455-9ad6-1c3c26ff77d1\", \"8adc625f-4f9d-4e6c-81a1-e30d75b290b1\", \"26b1a299-00ed-4032-baa8-63a4866b0706\", \"acfab9d4-1a8a-4f0e-ad8f-89cf4275fa6a\", \"0bea365b-4f06-400e-b584-ac1441bb923a\", \"15f27c14-eb82-4bc9-9cad-c0399725a35a\", \"b9dbadf0-c67c-4d4b-b63f-ed649d406482\", \"0b536c58-a39a-4cd2-b425-62858761dfd7\", \"74dd094c-f81d-49bb-b150-455280d667bf\", \"c9b21dd0-7179-4dad-893b-265f71b58eff\", \"311161e4-77bd-4357-93a1-fe774ad9e726\", \"20b3afa9-5c89-43cb-b4b4-8e88e3e243e2\", \"49e310c8-e505-4190-8e78-d0c49930e2cf\", \"abc48ffc-c952-4c05-a0d4-bc334eca5fef\", \"306d00ed-4bc0-46b0-b7a4-5b81bf7a5d61\", \"5b6562ce-e132-4419-af15-055ee8caaa0c\", \"1860b597-ebfa-49c0-96cb-7b408d614002\", \"04547465-2a33-489d-bc87-54251fb1cce7\", \"5898ddc7-a460-447d-97b3-fb42903ccdc6\", \"bb87180a-6077-4073-ad8c-6cca5707ef8c\", \"e2e09b4f-ee70-46f8-afd1-63dcfa135b5d\", \"d1dcaba6-5ec3-43d1-81ee-b64d2b5a158b\", \"e75270dd-012b-482e-8da2-d211625b6485\", \"e2010fd0-42a4-49d5-be37-bf1f94df7eb2\", \"2afdc560-8692-4bc1-979a-d163c0b30cbc\", \"6da1e5bd-9f8f-42a7-8057-a246b34c4fc9\", \"87ff5c96-cb0e-48cd-a246-2f06b5b7cce6\", \"3904fadc-bf21-42c7-83bc-58448a7abcf0\", \"6094cff2-5452-4f7f-9920-e8d3171c0136\", \"ba5e313a-595f-4fc8-a8ca-66b763255c6d\", \"41985322-75a3-46b1-b4bc-c58fe09f8376\", \"bac14675-bcdb-4e8b-841d-0787735a536a\", \"1b051003-7251-4774-97a5-4b5649f55199\", \"1cc50872-4eaa-426d-a5fe-2f1d0ac23ff9\", \"e2c33195-11be-4220-ad8e-4a0ed5388f20\", \"301b5b55-42b8-49ae-8b27-d324b1382aa8\", \"61ab2f4b-c66d-4a13-bdb8-b4e1e779452c\", \"90fd0245-6408-42ee-a93f-f6ad2f3a5b44\", \"e61068f0-62bd-4e1b-9f18-d3eebc28eabd\", \"34c9c349-d5b4-47be-a66e-9454aad46056\", \"0e8cda0d-0e69-492f-ad52-6741a6f182d1\", \"d16789ed-d71f-411e-a152-e89d891e1d71\", \"091c706e-06b7-41bc-96b4-204527e5109e\", \"f9f6f2ae-b332-4daa-9750-c98f42887a65\", \"a5175dcd-a946-432e-adf0-589a8e44ee48\", \"92ffa98a-43df-4cf8-8d88-49ab56171911\"], \"seller_profile_viewed\": false, \"briefs_per_week\": 50, \"send_weekly_email\": true, \"job_title\": \"COO\"}}, \"prospect_info\": {\"prospect_name\": \"CompuSystems\", \"prospect_domain\": \"compusystems.com\", \"prospect_contact\": {\"id\": \"8b3fc826-bea2-4f8c-b10d-83dc33307f79\", \"created_at\": \"2025-01-13T18:57:49.181000\", \"updated_at\": \"2025-01-13T18:57:49.181000\", \"prospect_request_id\": \"92ffa98a-43df-4cf8-8d88-49ab56171911\", \"name\": \"Mark Logiurato\", \"title\": \"CEO\", \"tenure_in_months\": 0, \"previous_employer\": null, \"previous_title\": null, \"email\": \"mark.logiurato@csireg.com\", \"linked_in_url\": null, \"buying_role\": null, \"persona_group_id\": \"ad559856-ae94-4688-8f56-82decf060dae\", \"external_source\": null}}}",
                "template_name": "pitch_2",
                "instructions": "Generate microsite content that presents the sales pitch in a clear, compelling, and attractive manner to the buyer persona at the prospect company. Use the provided context to create a persuasive narrative that aligns with the prospect's goals, pain points, and industry trends.\n\nThe content should not only focus on explaining why this is the ideal time and solution for them, but also emphasize the transformative impact of adopting the use case and how it solves the prospect's problems.\n\nThis microsite will be sent by the seller company to the prospect company.",
                "parent_id": null,
                "static_context": {
                    "microsite_analytics_context": {
                        "contact_email": "abe.durrant@avaratak.com",
                        "prospect_company_name": "CompuSystems",
                        "prospect_company_id": "92ffa98a-43df-4cf8-8d88-49ab56171911"
                    },
                    "cta": {
                        "url": "https://avaratak.com/schedule",
                        "button_text": "Reach Out"
                    },
                    "user_customization": {
                        "brand_primary_color": null,
                        "brand_secondary_color": null,
                        "brand_primary_text_color": null,
                        "brand_secondary_text_color": null,
                        "pep_logo_show": true,
                        "brand_logo_url": null
                    }
                }
            }}
        };
        let template_parameters: WorkflowTemplateParameters = serde_json::from_value(parameters_json).unwrap();

        // create a workflow from a template
        let workflow_template = CreateWorkflowFromTemplateRequest {
            template_parameters,
            ..Default::default()
        };
        let req = test::TestRequest::post().uri("/api/jwt/workflow-from-template")
            .set_json(workflow_template.clone())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        let json_value = serde_json::from_slice(&body).unwrap();
        let workflow: WorkflowId = serde_json::from_value(json_value).unwrap();
        assert_eq!(status, StatusCode::OK);

        // run the workflow once
        let workflow_run = WorkflowExecutorRunArguments {
            workflow_id: workflow.workflow_id,
            action: WorkflowExecutorAction::Pull {
                node_name: Some("microsite".to_string()),
                n_items: 1
            },
            simple_output: None,
            with_sources: None,
            tenant_id: None
        };
        let req = test::TestRequest::post().uri("/api/jwt/workflow/run")
            .set_json(workflow_run.clone())
            .append_header((header::AUTHORIZATION, jwt_token.clone()))
            .to_request();
        let res = service.call(req).await.unwrap();
        let status = res.status();
        let body = res.into_body().try_into_bytes().unwrap();
        assert_eq!(status, StatusCode::OK);
    }
}
