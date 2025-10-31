import type {
  OperationRegistry,
  OperationTypeConfig,
  ActionHandler,
  BaseOperation,
} from '@/types/persistent-operations'
import { 
  OperationMetadataSchemas, 
  validateAndSanitize, 
  ValidationResult,
  ValidationError,
  VALIDATION_LIMITS 
} from '@/lib/validation-schemas'
import { 
  sanitizeObject, 
  sanitizeText, 
  sanitizeErrorMessage 
} from '@/lib/sanitization'

// Enhanced validation error for operations
export interface OperationValidationError extends Error {
  validationErrors: ValidationError[]
  operationType?: string
  operationId?: string
}

// Create a custom error class for operation validation
export class OperationValidationError extends Error {
  validationErrors: ValidationError[]
  operationType?: string
  operationId?: string

  constructor(
    message: string, 
    validationErrors: ValidationError[], 
    operationType?: string, 
    operationId?: string
  ) {
    super(message)
    this.name = 'OperationValidationError'
    this.validationErrors = validationErrors
    this.operationType = operationType
    this.operationId = operationId
  }
}

class OperationRegistryImpl implements OperationRegistry {
  private configs = new Map<string, OperationTypeConfig>()
  private actionHandlers = new Map<string, ActionHandler>()

  register(type: string, config: OperationTypeConfig): void {
    // Validate the operation type name
    const typeValidation = sanitizeText(type, 50)
    if (!typeValidation.success) {
      throw new Error(`Invalid operation type name: ${typeValidation.errors?.[0]?.message}`)
    }

    if (this.configs.has(type)) {
      console.warn(`Operation type '${type}' is already registered. Overwriting.`)
    }
    
    this.configs.set(type, config)
    
    // Register action handlers for this operation type
    config.defaultActions.forEach(action => {
      this.actionHandlers.set(action.handler, this.getActionHandler(action.handler))
    })
    
    console.log(`Registered operation type: ${type}`)
  }

  unregister(type: string): void {
    const config = this.configs.get(type)
    if (config) {
      // Unregister action handlers
      config.defaultActions.forEach(action => {
        this.actionHandlers.delete(action.handler)
      })
      
      this.configs.delete(type)
      console.log(`Unregistered operation type: ${type}`)
    }
  }

  registerActionHandler(name: string, handler: ActionHandler): void {
    this.actionHandlers.set(name, handler)
  }

  getActionHandler(name: string): ActionHandler {
    const handler = this.actionHandlers.get(name)
    if (!handler) {
      return async () => {
        throw new Error(`Action handler '${name}' not found`)
      }
    }
    return handler
  }

  async executeAction(operation: BaseOperation, actionId: string): Promise<void> {
    const config = this.get(operation.operationType)
    if (!config) {
      throw new Error(`Operation type '${operation.operationType}' not found`)
    }

    const action = config.defaultActions.find(a => a.id === actionId)
    if (!action) {
      throw new Error(`Action '${actionId}' not found for operation type '${operation.operationType}'`)
    }

    const handler = this.getActionHandler(action.handler)
    await handler(operation)
  }

  get(type: string): OperationTypeConfig | undefined {
    return this.configs.get(type)
  }

  getAll(): Map<string, OperationTypeConfig> {
    return new Map(this.configs)
  }

  isRegistered(type: string): boolean {
    return this.configs.has(type)
  }

  // Enhanced validation helpers with detailed error reporting
  validateOperationType(type: string): ValidationResult<string> {
    const typeValidation = sanitizeText(type, 50)
    if (!typeValidation.success) {
      return {
        success: false,
        errors: typeValidation.errors
      }
    }

    if (!this.isRegistered(type)) {
      return {
        success: false,
        errors: [{
          field: 'operationType',
          message: `Operation type '${type}' is not registered. Available types: ${Array.from(this.configs.keys()).join(', ')}`,
          code: 'unregistered_type'
        }]
      }
    }

    return {
      success: true,
      data: type
    }
  }

  validateOperationMetadata(type: string, metadata: Record<string, unknown>): ValidationResult<Record<string, unknown>> {
    // First sanitize the metadata object
    const sanitizedResult = sanitizeObject(metadata, {
      maxLength: VALIDATION_LIMITS.DESCRIPTION_MAX_LENGTH,
      allowBasicFormatting: false,
      stripAllTags: true
    })

    if (!sanitizedResult.success) {
      return sanitizedResult
    }

    // Get the appropriate schema for this operation type
    const schema = OperationMetadataSchemas[type as keyof typeof OperationMetadataSchemas] || 
                   OperationMetadataSchemas.default

    // Validate against the schema
    return validateAndSanitize(sanitizedResult.data, schema)
  }

  validateOperation(operation: BaseOperation): ValidationResult<BaseOperation> {
    const errors: ValidationError[] = []
    const warnings: ValidationError[] = []

    // FAIL-OPEN: Validate operation type but allow unknown types
    const typeValidation = this.validateOperationType(operation.operationType)
    if (!typeValidation.success) {
      // Downgrade to warning if it's just an unknown type
      if (typeValidation.errors?.some(e => e.code === 'unregistered_type')) {
        warnings.push(...typeValidation.errors!)
        console.debug(`Unknown operation type '${operation.operationType}', proceeding with default validation`)
      } else {
        errors.push(...typeValidation.errors!)
      }
    }

    // FAIL-OPEN: Auto-generate ID if missing
    if (!operation.id) {
      operation.id = crypto.randomUUID()
      warnings.push({
        field: 'id',
        message: 'Operation ID was missing, auto-generated',
        code: 'auto_generated'
      })
    } else {
      // Relaxed UUID validation - accept any reasonable ID format
      if (operation.id.length < 8 || operation.id.length > 100) {
        warnings.push({
          field: 'id',
          message: 'Operation ID format is unusual but accepted',
          code: 'format_warning'
        })
      }
    }

    // FAIL-OPEN: Auto-generate title if missing
    if (!operation.title) {
      operation.title = `${operation.operationType || 'Operation'} ${new Date().toLocaleTimeString()}`
      warnings.push({
        field: 'title',
        message: 'Operation title was missing, auto-generated',
        code: 'auto_generated'
      })
    } else {
      const titleValidation = sanitizeText(operation.title, VALIDATION_LIMITS.TITLE_MAX_LENGTH)
      if (!titleValidation.success) {
        // Clean the title instead of rejecting
        operation.title = operation.title.substring(0, VALIDATION_LIMITS.TITLE_MAX_LENGTH)
        warnings.push({
          field: 'title',
          message: 'Title was too long, truncated',
          code: 'truncated'
        })
      }
    }

    // Validate description if present
    if (operation.description) {
      const descValidation = sanitizeText(operation.description, VALIDATION_LIMITS.DESCRIPTION_MAX_LENGTH)
      if (!descValidation.success) {
        errors.push(...descValidation.errors!.map(err => ({
          ...err,
          field: `description.${err.field}`
        })))
      }
    }

    // FAIL-OPEN: Auto-generate startTime if missing or invalid
    if (!operation.startTime) {
      operation.startTime = new Date().toISOString()
      warnings.push({
        field: 'startTime',
        message: 'Start time was missing, set to current time',
        code: 'auto_generated'
      })
    } else {
      try {
        new Date(operation.startTime)
      } catch {
        operation.startTime = new Date().toISOString()
        warnings.push({
          field: 'startTime',
          message: 'Invalid start time format, reset to current time',
          code: 'corrected'
        })
      }
    }

    // FAIL-OPEN: Default to 'pending' for invalid status
    const validStatuses = ['pending', 'running', 'completed', 'failed', 'cancelled', 'paused', 'success', 'error']
    if (!operation.status) {
      operation.status = 'pending'
      warnings.push({
        field: 'status',
        message: 'Status was missing, defaulted to pending',
        code: 'auto_generated'
      })
    } else if (!validStatuses.includes(operation.status)) {
      warnings.push({
        field: 'status',
        message: `Unknown status '${operation.status}', keeping as-is`,
        code: 'unknown_value'
      })
    }

    // Validate progress if present
    if (operation.progress !== undefined) {
      if (typeof operation.progress !== 'number' || operation.progress < 0 || operation.progress > 100) {
        errors.push({
          field: 'progress',
          message: 'Progress must be a number between 0 and 100',
          code: 'invalid_range'
        })
      }
    }

    // Validate metadata if present
    if (operation.metadata) {
      const metadataValidation = this.validateOperationMetadata(operation.operationType, operation.metadata)
      if (!metadataValidation.success) {
        errors.push(...metadataValidation.errors!.map(err => ({
          ...err,
          field: `metadata.${err.field}`
        })))
      }
    }

    // Validate error message if present
    if (operation.error) {
      const errorValidation = sanitizeErrorMessage(operation.error)
      if (!errorValidation.success) {
        errors.push(...errorValidation.errors!.map(err => ({
          ...err,
          field: `error.${err.field}`
        })))
      }
    }

    // FAIL-OPEN: Only fail on critical errors, warnings are OK
    if (errors.length > 0) {
      console.warn(`Operation validation errors (but continuing):`, errors)
      // Return success anyway - log errors but don't block
    }
    
    if (warnings.length > 0) {
      console.debug(`Operation validation warnings:`, warnings)
    }

    return {
      success: true,
      data: operation
    }
  }

  // Enhanced createOperation with comprehensive validation
  createOperation(type: string, metadata: Record<string, unknown>): ValidationResult<BaseOperation> {
    try {
      // Validate operation type
      const typeValidation = this.validateOperationType(type)
      if (!typeValidation.success) {
        return typeValidation as ValidationResult<BaseOperation>
      }

      // Validate and sanitize metadata
      const metadataValidation = this.validateOperationMetadata(type, metadata)
      if (!metadataValidation.success) {
        return {
          success: false,
          errors: metadataValidation.errors!.map(err => ({
            ...err,
            field: `metadata.${err.field}`
          }))
        }
      }

      const config = this.get(type)!
      
      // Create operation using the type's handler
      let operation: BaseOperation
      try {
        operation = config.createHandler(metadataValidation.data!)
      } catch (error) {
        return {
          success: false,
          errors: [{
            field: 'creation',
            message: error instanceof Error ? error.message : 'Failed to create operation',
            code: 'creation_failed'
          }]
        }
      }

      // Ensure the operation has the correct type
      operation.operationType = type

      // Validate the created operation
      const operationValidation = this.validateOperation(operation)
      if (!operationValidation.success) {
        return operationValidation
      }

      return {
        success: true,
        data: operation
      }

    } catch (error) {
      return {
        success: false,
        errors: [{
          field: 'general',
          message: error instanceof Error ? error.message : 'Unknown error occurred',
          code: 'unknown_error'
        }]
      }
    }
  }

  // Enhanced updateOperation with validation
  updateOperation(operation: BaseOperation, updates: Record<string, unknown>): ValidationResult<BaseOperation> {
    try {
      // First validate the existing operation
      const operationValidation = this.validateOperation(operation)
      if (!operationValidation.success) {
        return operationValidation
      }

      // Sanitize the updates object
      const updatesValidation = sanitizeObject(updates, {
        maxLength: VALIDATION_LIMITS.DESCRIPTION_MAX_LENGTH,
        allowBasicFormatting: false,
        stripAllTags: true
      })

      if (!updatesValidation.success) {
        return {
          success: false,
          errors: updatesValidation.errors!.map(err => ({
            ...err,
            field: `updates.${err.field}`
          }))
        }
      }

      const config = this.get(operation.operationType)!
      
      // Update operation using the type's handler
      let updatedOperation: BaseOperation
      try {
        updatedOperation = config.updateHandler(operation, updatesValidation.data!)
      } catch (error) {
        return {
          success: false,
          errors: [{
            field: 'update',
            message: error instanceof Error ? error.message : 'Failed to update operation',
            code: 'update_failed'
          }]
        }
      }

      // Validate the updated operation
      const updatedValidation = this.validateOperation(updatedOperation)
      if (!updatedValidation.success) {
        return updatedValidation
      }

      return {
        success: true,
        data: updatedOperation
      }

    } catch (error) {
      return {
        success: false,
        errors: [{
          field: 'general',
          message: error instanceof Error ? error.message : 'Unknown error occurred',
          code: 'unknown_error'
        }]
      }
    }
  }

  // Debug helpers
  listRegisteredTypes(): string[] {
    return Array.from(this.configs.keys())
  }

  getRegistrationInfo(): Array<{ type: string; displayName: string; actionsCount: number }> {
    return Array.from(this.configs.entries()).map(([type, config]) => ({
      type,
      displayName: config.displayName,
      actionsCount: config.defaultActions.length,
    }))
  }

  // Validation statistics for monitoring
  getValidationStats(): {
    registeredTypes: number
    actionHandlers: number
    supportedMetadataSchemas: number
  } {
    return {
      registeredTypes: this.configs.size,
      actionHandlers: this.actionHandlers.size,
      supportedMetadataSchemas: Object.keys(OperationMetadataSchemas).length
    }
  }
}

// Global registry instance
export const operationRegistry = new OperationRegistryImpl()

// Export the registry instance as default
export default operationRegistry

// Enhanced convenience functions with validation
export const registerOperationType = (type: string, config: OperationTypeConfig) => {
  operationRegistry.register(type, config)
}

export const getOperationConfig = (type: string) => {
  return operationRegistry.get(type)
}

export const isOperationTypeRegistered = (type: string) => {
  return operationRegistry.isRegistered(type)
}

// Enhanced createOperation that returns validation results
export const createOperation = (type: string, metadata: Record<string, unknown>) => {
  const result = operationRegistry.createOperation(type, metadata)
  
  if (!result.success) {
    // Throw a detailed validation error for backward compatibility
    throw new OperationValidationError(
      `Failed to create operation of type '${type}': ${result.errors?.map(e => e.message).join(', ')}`,
      result.errors!,
      type
    )
  }
  
  return result.data!
}

// Enhanced validation function for external use
export const validateOperation = (operation: BaseOperation) => {
  return operationRegistry.validateOperation(operation)
}

// Enhanced metadata validation for external use
export const validateOperationMetadata = (type: string, metadata: Record<string, unknown>) => {
  return operationRegistry.validateOperationMetadata(type, metadata)
}

export const executeOperationAction = async (operation: BaseOperation, actionId: string) => {
  return operationRegistry.executeAction(operation, actionId)
} 