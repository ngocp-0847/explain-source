import * as React from 'react'
import { useFormContext } from 'react-hook-form'
import { cn } from '@/lib/utils'
import { Label } from '@/components/ui/label'

const Form = ({ ...props }: React.ComponentPropsWithoutRef<'form'>) => {
  return <form {...props} />
}

const FormField = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & {
    name: string
  }
>(({ name, className, ...props }, ref) => {
  const methods = useFormContext()

  return (
    <div ref={ref} className={cn('space-y-2', className)} {...props}>
      {props.children}
    </div>
  )
})
FormField.displayName = 'FormField'

const FormLabel = ({ className, ...props }: React.ComponentPropsWithoutRef<typeof Label>) => {
  return <Label className={className} {...props} />
}
FormLabel.displayName = 'FormLabel'

const FormItem = ({ className, ...props }: React.HTMLAttributes<HTMLDivElement>) => {
  return <div className={cn('space-y-2', className)} {...props} />
}
FormItem.displayName = 'FormItem'

export { Form, FormField, FormLabel, FormItem }

