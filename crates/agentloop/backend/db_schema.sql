--
-- PostgreSQL database dump
--

-- Dumped from database version 15.12 (Debian 15.12-1.pgdg120+1)
-- Dumped by pg_dump version 15.12 (Debian 15.12-1.pgdg120+1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: uuid-ossp; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS "uuid-ossp" WITH SCHEMA public;


--
-- Name: EXTENSION "uuid-ossp"; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION "uuid-ossp" IS 'generate universally unique identifiers (UUIDs)';


--
-- Name: trigger_set_timestamp(); Type: FUNCTION; Schema: public; Owner: localuser
--

CREATE FUNCTION public.trigger_set_timestamp() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
  NEW.updated_at = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$;


ALTER FUNCTION public.trigger_set_timestamp() OWNER TO localuser;

--
-- Name: update_updated_at_column(); Type: FUNCTION; Schema: public; Owner: localuser
--

CREATE FUNCTION public.update_updated_at_column() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.update_updated_at_column() OWNER TO localuser;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT now() NOT NULL,
    success boolean NOT NULL,
    checksum bytea NOT NULL,
    execution_time bigint NOT NULL
);


ALTER TABLE public._sqlx_migrations OWNER TO localuser;

--
-- Name: api_keys; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.api_keys (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    user_id uuid NOT NULL,
    key_hash text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    last_used_at timestamp with time zone
);


ALTER TABLE public.api_keys OWNER TO localuser;

--
-- Name: assets; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.assets (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    type character varying(50) NOT NULL,
    gcs_object_name text NOT NULL,
    url text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.assets OWNER TO localuser;

--
-- Name: collections; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.collections (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    metadata jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.collections OWNER TO localuser;

--
-- Name: creative_formats; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.creative_formats (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    width integer,
    height integer,
    metadata jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    creative_type text NOT NULL,
    json_schema jsonb
);


ALTER TABLE public.creative_formats OWNER TO localuser;

--
-- Name: creatives; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.creatives (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    collection_id uuid,
    style_id uuid,
    research_ids uuid[],
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    asset_ids uuid[],
    creative_format_id uuid NOT NULL,
    is_published boolean DEFAULT false NOT NULL,
    publish_url text,
    html_url text DEFAULT ''::text NOT NULL,
    screenshot_url text DEFAULT ''::text NOT NULL
);


ALTER TABLE public.creatives OWNER TO localuser;

--
-- Name: custom_creative_formats; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.custom_creative_formats (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    description text,
    width integer,
    height integer,
    metadata jsonb,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    creative_type text NOT NULL,
    json_schema jsonb
);


ALTER TABLE public.custom_creative_formats OWNER TO localuser;

--
-- Name: password_reset_tokens; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.password_reset_tokens (
    token text NOT NULL,
    user_id uuid NOT NULL,
    expires_at timestamp with time zone NOT NULL
);


ALTER TABLE public.password_reset_tokens OWNER TO localuser;

--
-- Name: requests; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.requests (
    id integer NOT NULL,
    url text,
    content_to_style text,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    finished_at timestamp with time zone,
    deleted_at timestamp with time zone,
    what_to_create text,
    compressed_style_website_content bytea,
    compressed_output_html bytea,
    status text,
    execution_time_ms integer,
    user_id uuid,
    visual_feedback boolean,
    credits_used integer,
    favourite boolean DEFAULT false NOT NULL
);


ALTER TABLE public.requests OWNER TO localuser;

--
-- Name: requests_id_seq; Type: SEQUENCE; Schema: public; Owner: localuser
--

CREATE SEQUENCE public.requests_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.requests_id_seq OWNER TO localuser;

--
-- Name: requests_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: localuser
--

ALTER SEQUENCE public.requests_id_seq OWNED BY public.requests.id;


--
-- Name: research; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.research (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    status character varying(50) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    title character varying(255) DEFAULT 'Untitled Research'::character varying NOT NULL,
    content text DEFAULT ''::text NOT NULL,
    sources text[] DEFAULT '{}'::text[] NOT NULL
);


ALTER TABLE public.research OWNER TO localuser;

--
-- Name: research_chat_messages; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.research_chat_messages (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    chat_id uuid NOT NULL,
    sender_type character varying(100) NOT NULL,
    message jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.research_chat_messages OWNER TO localuser;

--
-- Name: research_chats; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.research_chats (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    schema jsonb NOT NULL,
    memory jsonb
);


ALTER TABLE public.research_chats OWNER TO localuser;

--
-- Name: research_workflows; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.research_workflows (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    title text NOT NULL,
    payload jsonb NOT NULL,
    user_id uuid NOT NULL,
    chat_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.research_workflows OWNER TO localuser;

--
-- Name: research_workflows_id_seq; Type: SEQUENCE; Schema: public; Owner: localuser
--

CREATE SEQUENCE public.research_workflows_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.research_workflows_id_seq OWNER TO localuser;

--
-- Name: research_workflows_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: localuser
--

ALTER SEQUENCE public.research_workflows_id_seq OWNED BY public.research_workflows.id;


--
-- Name: styles; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.styles (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    html_url text NOT NULL,
    screenshot_url text NOT NULL
);


ALTER TABLE public.styles OWNER TO localuser;

--
-- Name: user_db_collection_items; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.user_db_collection_items (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_db_collection_id uuid NOT NULL,
    item_data jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.user_db_collection_items OWNER TO localuser;

--
-- Name: user_db_collections; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.user_db_collections (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_id uuid NOT NULL,
    name text NOT NULL,
    description text,
    schema_definition jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.user_db_collections OWNER TO localuser;

--
-- Name: users; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    email character varying(255) NOT NULL,
    password_hash character varying(255) NOT NULL,
    stripe_customer_id character varying(255),
    email_verified boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    verification_token character varying(255),
    token_expiry timestamp with time zone
);


ALTER TABLE public.users OWNER TO localuser;

--
-- Name: webflow_creatives; Type: TABLE; Schema: public; Owner: localuser
--

CREATE TABLE public.webflow_creatives (
    id uuid NOT NULL,
    research_ids uuid[],
    publish_url text NOT NULL,
    is_published boolean DEFAULT true NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);


ALTER TABLE public.webflow_creatives OWNER TO localuser;

--
-- Name: TABLE webflow_creatives; Type: COMMENT; Schema: public; Owner: localuser
--

COMMENT ON TABLE public.webflow_creatives IS 'Stores creatives published to Webflow with simplified structure';


--
-- Name: requests id; Type: DEFAULT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.requests ALTER COLUMN id SET DEFAULT nextval('public.requests_id_seq'::regclass);


--
-- Name: research_workflows id; Type: DEFAULT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.research_workflows ALTER COLUMN id SET DEFAULT nextval('public.research_workflows_id_seq'::regclass);


--
-- Name: _sqlx_migrations _sqlx_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT _sqlx_migrations_pkey PRIMARY KEY (version);


--
-- Name: api_keys api_keys_key_hash_key; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.api_keys
    ADD CONSTRAINT api_keys_key_hash_key UNIQUE (key_hash);


--
-- Name: api_keys api_keys_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.api_keys
    ADD CONSTRAINT api_keys_pkey PRIMARY KEY (id);


--
-- Name: assets assets_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.assets
    ADD CONSTRAINT assets_pkey PRIMARY KEY (id);


--
-- Name: collections collections_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.collections
    ADD CONSTRAINT collections_pkey PRIMARY KEY (id);


--
-- Name: creative_formats creative_formats_name_key; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.creative_formats
    ADD CONSTRAINT creative_formats_name_key UNIQUE (name);


--
-- Name: creative_formats creative_formats_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.creative_formats
    ADD CONSTRAINT creative_formats_pkey PRIMARY KEY (id);


--
-- Name: creatives creatives_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.creatives
    ADD CONSTRAINT creatives_pkey PRIMARY KEY (id);


--
-- Name: custom_creative_formats custom_creative_formats_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.custom_creative_formats
    ADD CONSTRAINT custom_creative_formats_pkey PRIMARY KEY (id);


--
-- Name: custom_creative_formats custom_creative_formats_user_id_name_key; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.custom_creative_formats
    ADD CONSTRAINT custom_creative_formats_user_id_name_key UNIQUE (user_id, name);


--
-- Name: password_reset_tokens password_reset_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_pkey PRIMARY KEY (token);


--
-- Name: requests requests_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.requests
    ADD CONSTRAINT requests_pkey PRIMARY KEY (id);


--
-- Name: research_chat_messages research_chat_messages_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.research_chat_messages
    ADD CONSTRAINT research_chat_messages_pkey PRIMARY KEY (id);


--
-- Name: research_chats research_chats_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.research_chats
    ADD CONSTRAINT research_chats_pkey PRIMARY KEY (id);


--
-- Name: research research_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.research
    ADD CONSTRAINT research_pkey PRIMARY KEY (id);


--
-- Name: research_workflows research_workflows_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.research_workflows
    ADD CONSTRAINT research_workflows_pkey PRIMARY KEY (id);


--
-- Name: styles styles_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.styles
    ADD CONSTRAINT styles_pkey PRIMARY KEY (id);


--
-- Name: user_db_collection_items user_db_collection_items_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.user_db_collection_items
    ADD CONSTRAINT user_db_collection_items_pkey PRIMARY KEY (id);


--
-- Name: user_db_collections user_db_collections_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.user_db_collections
    ADD CONSTRAINT user_db_collections_pkey PRIMARY KEY (id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: webflow_creatives webflow_creatives_pkey; Type: CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.webflow_creatives
    ADD CONSTRAINT webflow_creatives_pkey PRIMARY KEY (id);


--
-- Name: idx_api_keys_key_hash; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_api_keys_key_hash ON public.api_keys USING btree (key_hash);


--
-- Name: idx_api_keys_user_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_api_keys_user_id ON public.api_keys USING btree (user_id);


--
-- Name: idx_assets_user_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_assets_user_id ON public.assets USING btree (user_id);


--
-- Name: idx_collections_user_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_collections_user_id ON public.collections USING btree (user_id);


--
-- Name: idx_creatives_collection_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_creatives_collection_id ON public.creatives USING btree (collection_id);


--
-- Name: idx_creatives_research_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_creatives_research_id ON public.creatives USING btree (research_ids);


--
-- Name: idx_custom_creative_formats_user_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_custom_creative_formats_user_id ON public.custom_creative_formats USING btree (user_id);


--
-- Name: idx_password_reset_tokens_expires_at; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_password_reset_tokens_expires_at ON public.password_reset_tokens USING btree (expires_at);


--
-- Name: idx_password_reset_tokens_user_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_password_reset_tokens_user_id ON public.password_reset_tokens USING btree (user_id);


--
-- Name: idx_requests_created_at; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_requests_created_at ON public.requests USING btree (created_at);


--
-- Name: idx_requests_favourite; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_requests_favourite ON public.requests USING btree (favourite);


--
-- Name: idx_requests_user_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_requests_user_id ON public.requests USING btree (user_id);


--
-- Name: idx_research_chat_messages_chat_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_research_chat_messages_chat_id ON public.research_chat_messages USING btree (chat_id);


--
-- Name: idx_research_chats_user_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_research_chats_user_id ON public.research_chats USING btree (user_id);


--
-- Name: idx_research_user_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_research_user_id ON public.research USING btree (user_id);


--
-- Name: idx_research_workflows_chat_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_research_workflows_chat_id ON public.research_workflows USING btree (chat_id);


--
-- Name: idx_research_workflows_created_at; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_research_workflows_created_at ON public.research_workflows USING btree (created_at);


--
-- Name: idx_research_workflows_user_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_research_workflows_user_id ON public.research_workflows USING btree (user_id);


--
-- Name: idx_styles_user_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_styles_user_id ON public.styles USING btree (user_id);


--
-- Name: idx_user_db_collection_items_collection_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_user_db_collection_items_collection_id ON public.user_db_collection_items USING btree (user_db_collection_id);


--
-- Name: idx_user_db_collections_user_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_user_db_collections_user_id ON public.user_db_collections USING btree (user_id);


--
-- Name: idx_users_verification_token; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_users_verification_token ON public.users USING btree (verification_token);


--
-- Name: idx_webflow_creatives_id; Type: INDEX; Schema: public; Owner: localuser
--

CREATE INDEX idx_webflow_creatives_id ON public.webflow_creatives USING btree (id);


--
-- Name: user_db_collection_items set_user_db_collection_items_updated_at; Type: TRIGGER; Schema: public; Owner: localuser
--

CREATE TRIGGER set_user_db_collection_items_updated_at BEFORE UPDATE ON public.user_db_collection_items FOR EACH ROW EXECUTE FUNCTION public.trigger_set_timestamp();


--
-- Name: user_db_collections set_user_db_collections_updated_at; Type: TRIGGER; Schema: public; Owner: localuser
--

CREATE TRIGGER set_user_db_collections_updated_at BEFORE UPDATE ON public.user_db_collections FOR EACH ROW EXECUTE FUNCTION public.trigger_set_timestamp();


--
-- Name: assets update_assets_updated_at; Type: TRIGGER; Schema: public; Owner: localuser
--

CREATE TRIGGER update_assets_updated_at BEFORE UPDATE ON public.assets FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: collections update_collections_updated_at; Type: TRIGGER; Schema: public; Owner: localuser
--

CREATE TRIGGER update_collections_updated_at BEFORE UPDATE ON public.collections FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: creative_formats update_creative_formats_updated_at; Type: TRIGGER; Schema: public; Owner: localuser
--

CREATE TRIGGER update_creative_formats_updated_at BEFORE UPDATE ON public.creative_formats FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: creatives update_creatives_updated_at; Type: TRIGGER; Schema: public; Owner: localuser
--

CREATE TRIGGER update_creatives_updated_at BEFORE UPDATE ON public.creatives FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: custom_creative_formats update_custom_creative_formats_updated_at; Type: TRIGGER; Schema: public; Owner: localuser
--

CREATE TRIGGER update_custom_creative_formats_updated_at BEFORE UPDATE ON public.custom_creative_formats FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: research_chats update_research_chats_updated_at; Type: TRIGGER; Schema: public; Owner: localuser
--

CREATE TRIGGER update_research_chats_updated_at BEFORE UPDATE ON public.research_chats FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: research update_research_updated_at; Type: TRIGGER; Schema: public; Owner: localuser
--

CREATE TRIGGER update_research_updated_at BEFORE UPDATE ON public.research FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: research_workflows update_research_workflows_updated_at; Type: TRIGGER; Schema: public; Owner: localuser
--

CREATE TRIGGER update_research_workflows_updated_at BEFORE UPDATE ON public.research_workflows FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: styles update_styles_updated_at; Type: TRIGGER; Schema: public; Owner: localuser
--

CREATE TRIGGER update_styles_updated_at BEFORE UPDATE ON public.styles FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: users update_users_updated_at; Type: TRIGGER; Schema: public; Owner: localuser
--

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON public.users FOR EACH ROW EXECUTE FUNCTION public.update_updated_at_column();


--
-- Name: api_keys api_keys_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.api_keys
    ADD CONSTRAINT api_keys_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: assets assets_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.assets
    ADD CONSTRAINT assets_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: collections collections_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.collections
    ADD CONSTRAINT collections_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: creatives creatives_collection_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.creatives
    ADD CONSTRAINT creatives_collection_id_fkey FOREIGN KEY (collection_id) REFERENCES public.collections(id) ON DELETE CASCADE;


--
-- Name: creatives creatives_style_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.creatives
    ADD CONSTRAINT creatives_style_id_fkey FOREIGN KEY (style_id) REFERENCES public.styles(id) ON DELETE SET NULL;


--
-- Name: custom_creative_formats custom_creative_formats_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.custom_creative_formats
    ADD CONSTRAINT custom_creative_formats_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: password_reset_tokens password_reset_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.password_reset_tokens
    ADD CONSTRAINT password_reset_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: requests requests_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.requests
    ADD CONSTRAINT requests_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: research_chat_messages research_chat_messages_chat_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.research_chat_messages
    ADD CONSTRAINT research_chat_messages_chat_id_fkey FOREIGN KEY (chat_id) REFERENCES public.research_chats(id);


--
-- Name: research_chats research_chats_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.research_chats
    ADD CONSTRAINT research_chats_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: research research_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.research
    ADD CONSTRAINT research_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: research_workflows research_workflows_chat_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.research_workflows
    ADD CONSTRAINT research_workflows_chat_id_fkey FOREIGN KEY (chat_id) REFERENCES public.research_chats(id) ON DELETE CASCADE;


--
-- Name: research_workflows research_workflows_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.research_workflows
    ADD CONSTRAINT research_workflows_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: styles styles_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.styles
    ADD CONSTRAINT styles_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_db_collection_items user_db_collection_items_user_db_collection_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.user_db_collection_items
    ADD CONSTRAINT user_db_collection_items_user_db_collection_id_fkey FOREIGN KEY (user_db_collection_id) REFERENCES public.user_db_collections(id) ON DELETE CASCADE;


--
-- Name: user_db_collections user_db_collections_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: localuser
--

ALTER TABLE ONLY public.user_db_collections
    ADD CONSTRAINT user_db_collections_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

